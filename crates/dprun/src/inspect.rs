use crate::structs::GUID;
use bytes::{ByteOrder, Bytes, LittleEndian};
use std::mem;

fn read_guid(slice: &[u8]) -> GUID {
    let mut guid = [0; 16];
    guid.copy_from_slice(slice);
    unsafe { mem::transmute(guid) }
}

struct CmdId(u16);
impl std::fmt::Debug for CmdId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "0x{:x}", self.0)
    }
}

#[derive(Debug)]
struct ProtocolMessage {
    signature: String,
    version: u16,
    cmd: CmdId,
    body: Command,
}

#[derive(Debug)]
enum Command {
    EnumSessionsReply(String, GUID),
    EnumSessions(GUID, u32),
    RequestPlayerId(u32),
    RequestPlayerReply(u32),
    CreatePlayer(u32, String),
    DeletePlayer(u32),
    AddForwardRequest(u32, u32),
    Ping(u32, u32),
    PingReply(u32, u32),
    SuperEnumPlayersReply,
    PacketizedData(u32, u32),
    PacketizedMessage(Box<ProtocolMessage>),
    PacketizedAck,
    Other(Vec<u8>),
}

fn parse_cmd(cmd: u16, mut message: Bytes) -> Command {
    match cmd {
        0x01 => {
            let size = LittleEndian::read_u32(&message.split_to(4)) as usize;
            message.advance(4);
            let guid = read_guid(&message.split_to(16));
            message.advance(size - 24);
            let _name_offset = LittleEndian::read_u32(&message.split_to(4));
            let name = String::from_utf8_lossy(&message);
            Command::EnumSessionsReply(name.to_string(), guid)
        }
        0x02 => {
            let guid = read_guid(&message.split_to(16));
            let flags = LittleEndian::read_u32(&message.split_to(4));
            Command::EnumSessions(guid, flags)
        }
        0x05 => {
            let flags = LittleEndian::read_u32(&message.split_to(4));
            Command::RequestPlayerId(flags)
        }
        0x07 => {
            let new_id = LittleEndian::read_u32(&message.split_to(4));
            Command::RequestPlayerReply(new_id)
        }
        0x08 => {
            message.advance(20);
            message.advance(8);
            let id = LittleEndian::read_u32(&message.split_to(4));
            let name_len = LittleEndian::read_u32(&message.split_to(4)) as usize;
            message.advance(8 * 4);
            let name = String::from_utf8_lossy(&message[0..name_len]);
            Command::CreatePlayer(id, name.to_string())
        }
        0x0b => {
            message.advance(4);
            let id = LittleEndian::read_u32(&message.split_to(4));
            Command::DeletePlayer(id)
        }
        0x13 => {
            let to = LittleEndian::read_u32(&message.split_to(4));
            let new_player = LittleEndian::read_u32(&message.split_to(4));
            Command::AddForwardRequest(to, new_player)
        }
        0x16 => {
            let from = LittleEndian::read_u32(&message.split_to(4));
            let ticks = LittleEndian::read_u32(&message.split_to(4));
            Command::Ping(from, ticks)
        }
        0x17 => {
            let from = LittleEndian::read_u32(&message.split_to(4));
            let ticks = LittleEndian::read_u32(&message.split_to(4));
            Command::PingReply(from, ticks)
        }
        0x29 => Command::SuperEnumPlayersReply,
        0x30 => {
            message.advance(16);
            let index = LittleEndian::read_u32(&message.split_to(4));
            message.advance(8);
            let total = LittleEndian::read_u32(&message.split_to(4));
            if total == 1 {
                message.advance(8);
                Command::PacketizedMessage(Box::new(parse_message(message)))
            } else {
                Command::PacketizedData(index, total)
            }
        }
        0x31 => Command::PacketizedAck,
        _ => Command::Other(message.to_vec()),
    }
}

fn parse_message(mut message: Bytes) -> ProtocolMessage {
    let mut signature = [0; 4];
    signature.copy_from_slice(&message.split_to(4));
    let cmd = LittleEndian::read_u16(&message.split_to(2));
    let version = LittleEndian::read_u16(&message.split_to(2));
    let sub = parse_cmd(cmd, message);
    ProtocolMessage {
        signature: String::from_utf8_lossy(&signature).to_string(),
        version,
        cmd: CmdId(cmd),
        body: sub,
    }
}

pub fn print_network_message(mut message: Bytes) {
    let mut header = [0; 16];
    header.copy_from_slice(&message.split_to(16));
    let guid: GUID = unsafe { mem::transmute(header) };
    println!("[print_network_message] message from: {:?}", guid);
    println!("{:#?}", parse_message(message));
}
