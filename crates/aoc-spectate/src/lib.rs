use tokio::net::TcpStream;
use futures::Future;
use std::io::Write;
use std::net::SocketAddr;

pub struct SpectateHeader {
    game_name: String,
    file_type: String,
    player_name: String,
}

impl SpectateHeader {
    pub fn parse(packet: &[u8]) -> SpectateHeader {
        let mut iter = packet.iter();
        let game_name_end = iter.position(|c| *c == b'\0').unwrap_or(0);
        iter.next();
        let file_type_end = iter.position(|c| *c == b'\0').unwrap_or(0);
        iter.next();
        let player_name_end = iter.position(|c| *c == b'\0').unwrap_or(0);
        SpectateHeader {
            game_name: String::from_utf8_lossy(&packet[0..game_name_end]).to_string(),
            file_type: String::from_utf8_lossy(&packet[32..file_type_end]).to_string(),
            player_name: String::from_utf8_lossy(&packet[64..player_name_end]).to_string(),
        }
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
    source: TcpStream,
}

impl SpectateSession {
    pub fn connect_local() -> impl Future<Item = SpectateSession> {
        let addr = "127.0.0.1:53754".parse::<SocketAddr>().unwrap();
        let stream = TcpStream::connect(&addr);
        let header = vec![0; 256];

        stream
            .and_then(move |stream| tokio::io::read_exact(stream, header))
            .map(move |(stream, header)| {
                SpectateSession {
                    header: SpectateHeader::parse(&header),
                    source: stream,
                }
            })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
