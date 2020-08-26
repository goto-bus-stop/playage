use byteorder::{ReadBytesExt, LE};
use std::io::{self, Cursor, Read};
use uuid::Uuid;

pub type DPID = i32;

fn read_guid(mut read: impl Read) -> io::Result<Uuid> {
    let mut guid = [0; 16];
    read.read_exact(&mut guid)?;
    Ok(Uuid::from_bytes(guid))
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
        let mut cursor = Cursor::new(bytes);

        let _dpid = cursor.read_u32::<LE>().unwrap();
        let guid = read_guid(&mut cursor).unwrap();

        let flags = cursor.read_i32::<LE>().unwrap();

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
        let mut cursor = Cursor::new(bytes);
        let create = cursor.read_u8().unwrap() != 0;
        let return_status = cursor.read_u8().unwrap() != 0;
        let _padding = cursor.read_u16::<LE>().unwrap();
        let open_flags = cursor.read_i32::<LE>().unwrap();
        let session_flags = cursor.read_i32::<LE>().unwrap();
        Self {
            create,
            return_status,
            open_flags,
            session_flags,
        }
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
        let mut cursor = Cursor::new(bytes);

        let flags = cursor.read_i32::<LE>().unwrap();

        let receiver_id = match read_guid(&mut cursor).unwrap() {
            guid if guid == Uuid::nil() => None,
            guid => Some(guid),
        };
        let sender_id = read_guid(&mut cursor).unwrap();

        let system_message = cursor.read_i32::<LE>().unwrap() != 0;
        let message_size = cursor.read_i32::<LE>().unwrap();
        let mut message = vec![0; message_size as usize];
        cursor.read_exact(&mut message).unwrap();

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
        let mut cursor = Cursor::new(bytes);

        let reply_to = read_guid(&mut cursor).unwrap();

        let name_server_id = cursor.read_i32::<LE>().unwrap();
        let message_size = cursor.read_i32::<LE>().unwrap();
        let mut message = vec![0; message_size as usize];
        cursor.read_exact(&mut message).unwrap();

        Self {
            reply_to,
            name_server_id,
            message,
        }
    }
}
