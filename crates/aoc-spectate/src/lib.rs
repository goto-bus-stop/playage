//! Work with spectator streams from UserPatch 1.4+ for Age of Empires 2.
//!
//! It can connect to spectator streams on local or remote machines, and provides the basic data
//! manipulation tools to set up a custom spectator server proxy.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

use async_std::{io::Result, net::TcpStream};
use futures::prelude::*;
use std::{io::Write as SyncWrite, net::SocketAddr};

/// Alias for any type of readable stream.
type AnyStream = Box<dyn AsyncRead + Send + Unpin>;

/// Holder for metadata from the spectator stream header.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectateHeader {
    /// The name of the UserPatch mod.
    ///
    /// Passed in as the GAME=$game_name parameter.
    pub game_name: String,
    /// Extension for the recorded game file being spectated.
    pub file_type: String,
    /// Name of the player the stream is being received from.
    pub player_name: String,
}

impl SpectateHeader {
    /// Create a new spectator stream header with the given data.
    pub fn new(
        game_name: impl ToString,
        file_type: impl ToString,
        player_name: impl ToString,
    ) -> Self {
        Self {
            game_name: game_name.to_string(),
            file_type: file_type.to_string(),
            player_name: player_name.to_string(),
        }
    }

    /// Parse a spectator stream header from a 256-byte slice.
    ///
    /// ## Example
    /// ```rust
    /// use aoc_spectate::SpectateHeader;
    /// let test_header_bytes = b"age2_x1\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0mgz\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0Example Player\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    ///
    /// let actual = SpectateHeader::parse(test_header_bytes).unwrap();
    /// let expected = SpectateHeader::new("age2_x1", "mgz", "Example Player");
    /// assert_eq!(actual, expected);
    /// ```
    pub fn parse(packet: &[u8]) -> Result<Self> {
        assert_eq!(packet.len(), 256);
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

    /// Serialize the spectator stream header to a byte array.
    ///
    /// # Example
    /// ```rust
    /// use aoc_spectate::SpectateHeader;
    /// let test_header_bytes = b"age2_x1\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0mgz\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0Example Player\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    ///
    /// let header = SpectateHeader::new("age2_x1", "mgz", "Example Player");
    /// assert_eq!(test_header_bytes.to_vec(), header.to_vec());
    /// ```
    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(256);
        // Not like these can really fail
        SyncWrite::write_all(&mut bytes, self.game_name.as_bytes()).unwrap();
        bytes.extend(vec![0; 32 - bytes.len()]);
        SyncWrite::write_all(&mut bytes, self.file_type.as_bytes()).unwrap();
        bytes.extend(vec![0; 64 - bytes.len()]);
        SyncWrite::write_all(&mut bytes, self.player_name.as_bytes()).unwrap();
        bytes.extend(vec![0; 256 - bytes.len()]);
        bytes
    }
}

/// Wrapper for a byte stream from a UserPatch spectator server.
pub struct SpectateStream {
    header: SpectateHeader,
    source: AnyStream,
}

impl SpectateStream {
    /// Connect to a spectator stream on the local machine.
    #[inline]
    pub async fn connect_local() -> Result<SpectateStream> {
        let addr = "127.0.0.1:53754".parse::<SocketAddr>().unwrap();
        let stream = TcpStream::connect(&addr).await?;
        Self::connect_stream(Box::new(stream)).await
    }

    /// Wrap a spectator stream.
    #[inline]
    pub async fn connect_stream(mut stream: AnyStream) -> Result<SpectateStream> {
        let mut header = [0; 256];
        stream.read_exact(&mut header).await?;

        let header = SpectateHeader::parse(&header)?;
        Ok(SpectateStream {
            header,
            source: stream,
        })
    }

    /// Get the name of the UserPatch mod used to play the game being spectated.
    #[inline]
    pub fn game_name(&self) -> &str {
        &self.header.game_name
    }

    /// Get the file extension for the recorded game being spectated.
    #[inline]
    pub fn file_type(&self) -> &str {
        &self.header.file_type
    }

    /// Get the player name being spectated.
    #[inline]
    pub fn player_name(&self) -> &str {
        &self.header.player_name
    }

    /// Read the recorded game file header. Returns the header as a byte array.
    #[inline]
    pub async fn read_rec_header(&mut self) -> Result<Vec<u8>> {
        let mut size = [0; 4];
        self.source.read_exact(&mut size[..]).await?;
        let size = u32::from_le_bytes(size) as usize;

        let mut header: Vec<u8> = vec![0; size + 8];
        SyncWrite::write_all(&mut header, &(size as u32).to_le_bytes())?;
        self.source.read_exact(&mut header[4..]).await?;

        Ok(header)
    }

    /// Get the raw spectator stream by reference.
    #[inline]
    pub fn inner(&mut self) -> &mut AnyStream {
        &mut self.source
    }

    /// Get the raw spectator stream.
    #[inline]
    pub fn into_inner(self) -> AnyStream {
        self.source
    }
}
