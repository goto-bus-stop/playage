use byteorder::{ReadBytesExt, LE};
use std::io::{self, Cursor, Read};
use uuid::Uuid;

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
    EnumSessionsReply(String, Uuid),
    EnumSessions(Uuid, u32),
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

fn parse_cmd(cmd: u16, mut message: impl Read) -> io::Result<Command> {
    match cmd {
        0x01 => {
            let size = message.read_u32::<LE>()? as usize;
            let _ = message.read_u32::<LE>()?;
            let guid = {
                let mut bytes = [0; 16];
                message.read_exact(&mut bytes)?;
                Uuid::from_bytes(bytes)
            };
            std::io::copy(
                &mut message.by_ref().take((size - 24) as u64),
                &mut std::io::sink(),
            )?;
            let _name_offset = message.read_u32::<LE>()?;
            let mut name = vec![];
            message.read_to_end(&mut name)?;
            Ok(Command::EnumSessionsReply(
                String::from_utf8_lossy(&name).to_string(),
                guid,
            ))
        }
        0x02 => {
            let guid = {
                let mut bytes = [0; 16];
                message.read_exact(&mut bytes)?;
                Uuid::from_bytes(bytes)
            };
            let flags = message.read_u32::<LE>()?;
            Ok(Command::EnumSessions(guid, flags))
        }
        0x05 => {
            let flags = message.read_u32::<LE>()?;
            Ok(Command::RequestPlayerId(flags))
        }
        0x07 => {
            let new_id = message.read_u32::<LE>()?;
            Ok(Command::RequestPlayerReply(new_id))
        }
        0x08 => {
            std::io::copy(&mut message.by_ref().take(20), &mut std::io::sink())?;
            std::io::copy(&mut message.by_ref().take(8), &mut std::io::sink())?;
            let id = message.read_u32::<LE>()?;
            let name_len = message.read_u32::<LE>()? as usize;
            std::io::copy(&mut message.by_ref().take(8 * 4), &mut std::io::sink())?;
            let name = {
                let mut name_bytes = vec![0; name_len];
                message.read_exact(&mut name_bytes)?;
                String::from_utf8_lossy(&name_bytes).to_string()
            };
            Ok(Command::CreatePlayer(id, name))
        }
        0x0b => {
            let _ = message.read_u32::<LE>()?;
            let id = message.read_u32::<LE>()?;
            Ok(Command::DeletePlayer(id))
        }
        0x13 => {
            let to = message.read_u32::<LE>()?;
            let new_player = message.read_u32::<LE>()?;
            Ok(Command::AddForwardRequest(to, new_player))
        }
        0x16 => {
            let from = message.read_u32::<LE>()?;
            let ticks = message.read_u32::<LE>()?;
            Ok(Command::Ping(from, ticks))
        }
        0x17 => {
            let from = message.read_u32::<LE>()?;
            let ticks = message.read_u32::<LE>()?;
            Ok(Command::PingReply(from, ticks))
        }
        0x29 => Ok(Command::SuperEnumPlayersReply),
        0x30 => {
            std::io::copy(&mut message.by_ref().take(16), &mut std::io::sink())?;
            let index = message.read_u32::<LE>()?;
            std::io::copy(&mut message.by_ref().take(8), &mut std::io::sink())?;
            let total = message.read_u32::<LE>()?;
            if total == 1 {
                std::io::copy(&mut message.by_ref().take(8), &mut std::io::sink())?;
                Ok(Command::PacketizedMessage(Box::new(parse_message(
                    message,
                )?)))
            } else {
                Ok(Command::PacketizedData(index, total))
            }
        }
        0x31 => Ok(Command::PacketizedAck),
        _ => {
            let mut bytes = vec![];
            message.read_to_end(&mut bytes)?;
            Ok(Command::Other(bytes))
        }
    }
}

fn parse_message(mut message: impl Read) -> io::Result<ProtocolMessage> {
    let mut signature = [0; 4];
    message.read_exact(&mut signature)?;
    let cmd = message.read_u16::<LE>()?;
    let version = message.read_u16::<LE>()?;
    let sub = parse_cmd(cmd, message)?;
    Ok(ProtocolMessage {
        signature: String::from_utf8_lossy(&signature).to_string(),
        version,
        cmd: CmdId(cmd),
        body: sub,
    })
}

pub fn print_network_message(message: &[u8]) {
    let mut message = Cursor::new(message);
    let guid = {
        let mut bytes = [0; 16];
        message.read_exact(&mut bytes).unwrap();
        Uuid::from_bytes(bytes)
    };
    println!("[print_network_message] message from: {:?}", guid);
    println!("{:#?}", parse_message(message).unwrap());
}
