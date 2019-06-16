use bytes::Buf;
use std::fmt::{Debug, Display, Formatter, Result as FormatResult};
use std::io::Cursor;
use std::mem;
use uuid::Uuid;

pub type DPID = i32;

fn read_guid(read: &mut Buf) -> Uuid {
    let mut guid = [0; 16];
    read.copy_to_slice(&mut guid);
    Uuid::from_bytes(guid)
}

#[derive(Debug)]
#[repr(C)]
pub struct CreatePlayerData {
    // pub player_id: DPID,
    pub player_guid: Uuid,
    pub flags: i32,
}

impl CreatePlayerData {
    pub fn parse(bytes: &[u8]) -> Self {
        let mut read = Cursor::new(bytes);

        let _dpid = read.get_i32_le();
        let guid = read_guid(&mut read);

        let flags = read.get_i32_le();

        Self {
            // player_id: dpid,
            player_guid: guid,
            flags,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct EnumSessionsData {
    pub message: Vec<u8>,
}

#[derive(Debug)]
#[repr(C)]
pub struct OpenData {
    pub create: bool,
    pub return_status: bool,
    pub open_flags: i32,
    pub session_flags: i32,
}

impl OpenData {
    pub fn parse(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), mem::size_of::<Self>());
        let mut buffer = [0; mem::size_of::<Self>()];
        buffer.copy_from_slice(bytes);
        let cast: Self = unsafe { mem::transmute(buffer) };
        cast
    }
}

#[derive(Debug)]
pub struct SendData {
    pub flags: i32,
    pub receiver_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub system_message: bool,
    pub message: Vec<u8>,
}

impl SendData {
    pub fn parse(bytes: &[u8]) -> Self {
        let mut read = Cursor::new(bytes);

        let flags = read.get_i32_le();

        let receiver_id = match read_guid(&mut read) {
            guid if guid == Uuid::nil() => None,
            guid => Some(guid),
        };
        let sender_id = read_guid(&mut read);

        let system_message = read.get_i32_le() != 0;
        let message_size = read.get_i32_le();
        let mut message = vec![0; message_size as usize];
        read.copy_to_slice(&mut message);

        Self {
            flags,
            receiver_id,
            sender_id,
            system_message,
            message,
        }
    }
}

#[derive(Debug)]
pub struct ReplyData {
    pub reply_to: Uuid,
    pub name_server_id: DPID,
    pub message: Vec<u8>,
}

impl ReplyData {
    pub fn parse(bytes: &[u8]) -> Self {
        let mut read = Cursor::new(bytes);

        let mut reply_to = read_guid(&mut read);

        let name_server_id = read.get_i32_le();
        let message_size = read.get_i32_le();
        let mut message = vec![0; message_size as usize];
        read.copy_to_slice(&mut message);

        Self {
            reply_to,
            name_server_id,
            message,
        }
    }
}
