use bytes::Buf;
use std::io::Cursor;

pub type DPID = i32;

#[derive(Debug)]
#[repr(C)]
struct DPCaps {
    /// Size of structure in bytes
    size: i32,
    flags: i32,
    max_buffer_size: i32,
    /// Obsolete.
    max_queue_size: i32,
    /// Maximum players/groups (local + remote)
    max_players: i32,
    /// Bandwidth in 100 bits per second units: i32,
    /// i.e. 24 is 2400, 96 is 9600, etc.
    hundred_baud: i32,
    /// Estimated latency: i32, 0 = unknown
    latency: i32,
    /// Maximum # of locally created players
    max_local_players: i32,
    /// Maximum header length in bytes
    header_length: i32,
    /// Service provider's suggested timeout value
    /// This is how long DirectPlay will wait for
    /// responses to system messages
    timeout: i32,
}

#[derive(Debug)]
#[repr(C)]
struct AddPlayerToGroupData {
  player_id: DPID,
  group_id: DPID,
}

#[derive(Debug)]
#[repr(C)]
struct CloseData {
}

#[derive(Debug)]
#[repr(C)]
struct CreateGroupData {
  group_id: DPID,
  flags: i32,
  message_header: Vec<u8>,
}

#[derive(Debug)]
#[repr(C)]
pub struct CreatePlayerData {
  pub player_id: DPID,
  pub flags: i32,
}

#[derive(Debug)]
#[repr(C)]
struct DeleteGroupData {
  group_id: DPID,
  flags: i32,
}

#[derive(Debug)]
#[repr(C)]
struct DeletePlayerData {
  player_id: DPID,
  flags: i32,
}

#[derive(Debug)]
#[repr(C)]
pub struct EnumSessionsData {
  pub message: Vec<u8>,
}

#[repr(C)]
struct GetAddressData {
  player_id: DPID,
  flags: i32,
  // LPDPADDRESS    lpAddress,
  // LPDWORD        lpdwAddressSize,
}

#[repr(C)]
struct GetAddressChoicesData {
  // LPDPADDRESS    lpAddress,
  // LPDWORD        lpdwAddressSize,
}

#[repr(C)]
struct GetCapsData {
  player_id: DPID,
  caps: DPCaps,
  flags: i32,
}

#[derive(Debug)]
#[repr(C)]
pub struct OpenData {
  pub create: bool,
  pub return_status: bool,
  pub open_flags: i32,
  pub session_flags: i32,
}

#[derive(Debug)]
#[repr(C)]
struct RemovePlayerFromGroupData {
  player_id: DPID,
  group_id: DPID,
}

#[derive(Debug)]
pub struct ReplyData {
    pub reply_to_id: DPID,
    pub name_server_id: DPID,
    pub message: Vec<u8>,
}

impl ReplyData {
    pub fn parse(bytes: &[u8]) -> Self {
        let mut read = Cursor::new(bytes);
        let reply_to_id = read.get_i32_le();
        let name_server_id = read.get_i32_le();
        let message_size = read.get_u32_le();
        let mut message = vec![0; message_size as usize];
        read.copy_to_slice(&mut message);

        Self {
            reply_to_id,
            name_server_id,
            message: message,
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct SendData {
  flags: i32,
  to_player_id: DPID,
  from_player_id: DPID,
  message: Vec<u8>,
  is_system_message: bool,
}

#[derive(Debug)]
#[repr(C)]
struct SendToGroupData {
  flags: i32,
  to_group_id: DPID,
  from_player_id: DPID,
  message: Vec<u8>,
}

#[derive(Debug)]
#[repr(C)]
struct SendExData {
  flags: i32,
  to_player_id: DPID,
  from_player_id: DPID,
  send_buffers: Vec<u8>,
  priority: i32,
  timeout: i32,
  context: Vec<u8>,
  // LPDWORD        lpdwSPMsgID,
  is_system_message: bool,
}

#[derive(Debug)]
#[repr(C)]
struct SendToGroupExData {
  flags: i32,
  to_group_id: DPID,
  from_player_id: DPID,
  send_buffers: Vec<u8>,
  priority: i32,
  timeout: i32,
  context: Vec<u8>,
  // LPDWORD        lpdwSPMsgID,
}

#[derive(Debug)]
#[repr(C)]
struct GetMessageQueueData {
  flags: i32,
  from_id: DPID,
  to_id: DPID,
  // LPDWORD        lpdwNumMsgs,
  // LPDWORD        lpdwNumBytes,
}

#[derive(Debug)]
#[repr(C)]
struct CancelData {
  flags: i32,
  // LPRGLPVOID     lprglpvSPMsgID,
  sp_message_id: i32,
  min_priority: i32,
  max_priority: i32,
}

#[derive(Debug)]
#[repr(C)]
struct ShutdownData {
}
