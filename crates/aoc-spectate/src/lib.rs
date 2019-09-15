use async_std::{io::Result, net::TcpStream};
use futures::prelude::*;
use std::{io::Write as _, net::SocketAddr};

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
        std::io::Write::write_all(&mut bytes, self.game_name.as_bytes()).unwrap();
        bytes.extend(vec![0; 32 - bytes.len()]);
        std::io::Write::write_all(&mut bytes, self.file_type.as_bytes()).unwrap();
        bytes.extend(vec![0; 64 - bytes.len()]);
        std::io::Write::write_all(&mut bytes, self.player_name.as_bytes()).unwrap();
        bytes.extend(vec![0; 256 - bytes.len()]);
        bytes
    }
}

pub struct SpectateSession {
    header: SpectateHeader,
    source: Box<dyn AsyncRead + Send + Unpin>,
}

impl SpectateSession {
    pub async fn connect_local() -> Result<SpectateSession> {
        let addr = "127.0.0.1:53754".parse::<SocketAddr>().unwrap();
        let stream = TcpStream::connect(&addr).await?;
        Self::connect_stream(Box::new(stream)).await
    }

    pub async fn connect_stream(
        mut stream: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<SpectateSession> {
        let mut header = [0; 256];
        stream.read_exact(&mut header).await?;

        let header = SpectateHeader::parse(&header)?;
        Ok(SpectateSession {
            header,
            source: stream,
        })
    }

    pub fn game_name(&self) -> &str {
        &self.header.game_name
    }

    pub fn file_type(&self) -> &str {
        &self.header.file_type
    }

    pub fn player_name(&self) -> &str {
        &self.header.player_name
    }

    pub async fn read_rec_header(&mut self) -> Result<(usize, Vec<u8>)> {
        let mut size = [0; 4];
        self.source.read_exact(&mut size[..]).await?;
        let size = u32::from_le_bytes(size) as usize;

        let mut header: Vec<u8> = vec![0; size + 4];
        self.source.read_exact(&mut header).await?;

        Ok((size, header))
    }

    pub fn stream(&mut self) -> &mut Box<dyn AsyncRead + Send + Unpin> {
        &mut self.source
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
