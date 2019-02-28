use tokio::net::TcpStream;
use tokio::prelude::*;
use std::io::{Write, Result};
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct SpectateHeader {
    game_name: String,
    file_type: String,
    player_name: String,
}

impl SpectateHeader {
    pub fn new(game_name: &str, file_type: &str, player_name: &str) -> Self {
        Self {
            game_name: game_name.to_string(),
            file_type: file_type.to_string(),
            player_name: player_name.to_string(),
        }
    }

    pub fn parse(packet: &[u8]) -> Result<Self> {
        let mut iter = packet.iter();
        let game_name_end = iter.position(|c| *c == b'\0').unwrap_or(0);
        let mut iter = iter.skip(31 - game_name_end);
        let file_type_end = iter.position(|c| *c == b'\0').unwrap_or(0);
        let mut iter = iter.skip(31 - file_type_end);
        let player_name_end = iter.position(|c| *c == b'\0').unwrap_or(0);

        Ok(Self {
            game_name: String::from_utf8_lossy(&packet[0..game_name_end]).to_string(),
            file_type: String::from_utf8_lossy(&packet[32..32 + file_type_end]).to_string(),
            player_name: String::from_utf8_lossy(&packet[64..64 + player_name_end]).to_string(),
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(256);
        // Not like these can really fail
        bytes.write_all(self.game_name.as_bytes()).unwrap();
        bytes.extend(vec![0; 32 - bytes.len()]);
        bytes.write_all(self.file_type.as_bytes()).unwrap();
        bytes.extend(vec![0; 64 - bytes.len()]);
        bytes.write_all(self.player_name.as_bytes()).unwrap();
        bytes.extend(vec![0; 256 - bytes.len()]);
        bytes
    }
}

pub struct SpectateSession {
    header: SpectateHeader,
    source: Box<dyn AsyncRead>,
}

pub type SpectateSessionFuture = impl Future<Item = SpectateSession, Error = std::io::Error>;

impl SpectateSession {
    pub fn connect_local() -> SpectateSessionFuture {
        let addr = "127.0.0.1:53754".parse::<SocketAddr>().unwrap();
        let stream = TcpStream::connect(&addr);
        stream.and_then(move |stream| Self::connect_stream(Box::new(stream)))
    }

    pub fn connect_stream(stream: Box<dyn AsyncRead>) -> SpectateSessionFuture {
        let header = vec![0; 256];

        tokio::io::read_exact(stream, header).and_then(move |(stream, header)| {
            future::result(SpectateHeader::parse(&header)).map(move |header| {
                SpectateSession { header, source: stream }
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::SpectateHeader;

    #[test]
    fn parse_header() {
        let test_header_bytes = b"age2_x1\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0mgz\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0Example Player\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

        let actual = SpectateHeader::parse(test_header_bytes).unwrap();
        let expected = SpectateHeader::new("age2_x1", "mgz", "Example Player");
        assert_eq!(format!("{:?}", actual), format!("{:?}", expected));
    }

    #[test]
    fn serialize_header() {
        let test_header_bytes = b"age2_x1\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0mgz\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0Example Player\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

        let header = SpectateHeader::new("age2_x1", "mgz", "Example Player");
        assert_eq!(test_header_bytes.to_vec(), header.to_vec());
    }
}
