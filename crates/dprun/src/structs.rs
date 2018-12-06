use bytes::Buf;
use std::mem;
use std::fmt::{Formatter, Debug, Display, Result as FormatResult};
use std::io::Cursor;

pub type DPID = i32;

/// GUID structure, for identifying DirectPlay interfaces, applications, and address types.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GUID(pub u32, pub u16, pub u16, pub u8, pub u8, pub u8, pub u8, pub u8, pub u8, pub u8, pub u8);

impl Display for GUID {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        write!(f, "{{{:08X?}-{:04X?}-{:04x?}-{:02X?}{:02X?}-{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?}}}",
               self.0,
               self.1,
               self.2,
               self.3, self.4,
               self.5, self.6, self.7, self.8, self.9, self.10)
    }
}

impl Debug for GUID {
    fn fmt(&self, f: &mut Formatter) -> FormatResult {
        write!(f, "GUID({:08X?}, {:04X?}, {:04x?}, {:02X?}{:02X?}, {:02X?}{:02X?}{:02X?}{:02X?}{:02X?}{:02X?})",
               self.0,
               self.1,
               self.2,
               self.3, self.4,
               self.5, self.6, self.7, self.8, self.9, self.10)
    }
}

fn read_guid(read: &mut Buf) -> GUID {
    let mut guid = [0; 16];
    read.copy_to_slice(&mut guid);
    unsafe { mem::transmute(guid) }
}

#[derive(Debug)]
#[repr(C)]
pub struct CreatePlayerData {
  // pub player_id: DPID,
  pub player_guid: GUID,
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

const GUID_NULL: GUID = GUID(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);

#[derive(Debug)]
pub struct SendData {
    pub flags: i32,
    pub receiver_id: Option<GUID>,
    pub sender_id: GUID,
    pub system_message: bool,
    pub message: Vec<u8>,
}

impl SendData {
    pub fn parse(bytes: &[u8]) -> Self {
        let mut read = Cursor::new(bytes);

        let flags = read.get_i32_le();

        let receiver_id = match read_guid(&mut read) {
            GUID_NULL => None,
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
    pub reply_to: GUID,
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
            message: message,
        }
    }
}
