use getset::Getters;
use protobuf::CodedInputStream;

use crate::error::{HeaderParsingError, Result};
use crate::reader::ext::ReadExt;

const MAX_OS_PATH: usize = 260;

/// Expected demo type.
const EXPECTED_DEMO_TYPE: &str = "HL2DEMO"; // in UPPERCASE
/// Expected demo protocol.
const EXPECTED_DEMO_PROTOCOL: u32 = 4;
/// Expected game name.
const EXPECTED_GAME: &str = "csgo"; // in lowercase

#[derive(Getters, Debug)]
#[getset(get = "pub")]
pub struct DemoHeader {
    /// Demo type. Should always be `HL2DEMO`.
    demo_type: String,
    /// Demo protocol version. Should always be `4`.
    demo_protocol: u32,
    /// Network protocol version.
    network_protocol: u32,
    /// Name of the server on which the game was played.
    server_name: String,
    /// Name of the client. _Almost_ always `GOTV Demo`.
    client_name: String,
    /// Name of the map on which the game was played.
    map_name: String,
    /// Name of the game. Should always be `csgo`.
    game: String,
    /// Duration of the game, in seconds.
    duration: f32,
    /// Total number of ticks.
    ticks: u32,
    /// Total number of frames.
    frames: u32,
    /// Length of Signon data, in bytes.
    signon_length: u32,
}

impl DemoHeader {
    pub(crate) fn try_new(reader: &mut CodedInputStream) -> Result<Self> {
        let demo_type = reader.read_string_limited(8)?;
        if demo_type != EXPECTED_DEMO_TYPE {
            return Err(HeaderParsingError::InvalidDemoType {
                expected: EXPECTED_DEMO_TYPE,
                found: demo_type,
            }
            .into());
        }

        let demo_protocol = reader.read_fixed32()?;
        if demo_protocol != EXPECTED_DEMO_PROTOCOL {
            return Err(HeaderParsingError::InvalidDemoProtocol {
                expected: EXPECTED_DEMO_PROTOCOL,
                found: demo_protocol,
            }
            .into());
        }

        let network_protocol = reader.read_fixed32()?;
        let server_name = reader.read_string_limited(MAX_OS_PATH)?;
        let client_name = reader.read_string_limited(MAX_OS_PATH)?;
        let map_name = reader.read_string_limited(MAX_OS_PATH)?;

        let game = reader.read_string_limited(MAX_OS_PATH)?;
        if game != EXPECTED_GAME {
            return Err(HeaderParsingError::InvalidGame {
                expected: EXPECTED_GAME,
                found: game,
            }
            .into());
        }

        Ok(Self {
            demo_type,
            demo_protocol,
            network_protocol,
            server_name,
            client_name,
            map_name,
            game,
            duration: reader.read_float()?,
            ticks: reader.read_fixed32()?,
            frames: reader.read_fixed32()?,
            signon_length: reader.read_fixed32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::DATA_TESTS_DIR;
    use const_format::concatcp;

    use crate::error::{Error, HeaderParsingError};
    use protobuf::CodedInputStream;
    use std::fs::File;

    #[test]
    fn valid_header() {
        let mut file = File::open(concatcp!(DATA_TESTS_DIR, "/valid_header.dem")).unwrap();
        let mut buf = CodedInputStream::new(&mut file);
        let header = DemoHeader::try_new(&mut buf).unwrap();
        assert_eq!(header.network_protocol, 13848);
        assert_eq!(
            header.server_name,
            String::from("FACEIT - register to play here")
        );
        assert_eq!(header.client_name, String::from("GOTV Demo"));
        assert_eq!(header.map_name, String::from("de_anubis"));
        assert_eq!(header.duration, 3338.0);
        assert_eq!(header.ticks, 427351);
        assert_eq!(header.frames, 426531);
        assert_eq!(header.signon_length, 582290);
    }

    #[test]
    fn error_on_invalid_demo_type() {
        let mut file =
            File::open(concatcp!(DATA_TESTS_DIR, "/header_invalid_demo_type.dem")).unwrap();
        let mut buf = CodedInputStream::new(&mut file);
        let res = DemoHeader::try_new(&mut buf);
        assert!(matches!(
            res,
            Err(Error::HeaderParsing(
                HeaderParsingError::InvalidDemoType { .. }
            ))
        ));
    }

    #[test]
    fn error_on_invalid_demo_protocol() {
        let mut file = File::open(concatcp!(
            DATA_TESTS_DIR,
            "/header_invalid_demo_protocol.dem"
        ))
        .unwrap();
        let mut buf = CodedInputStream::new(&mut file);
        let res = DemoHeader::try_new(&mut buf);
        assert!(matches!(
            res,
            Err(Error::HeaderParsing(
                HeaderParsingError::InvalidDemoProtocol { .. }
            ))
        ));
    }

    #[test]
    fn error_on_invalid_game() {
        let mut file = File::open(concatcp!(DATA_TESTS_DIR, "/header_invalid_game.dem")).unwrap();
        let mut buf = CodedInputStream::new(&mut file);
        let res = DemoHeader::try_new(&mut buf);
        assert!(matches!(
            res,
            Err(Error::HeaderParsing(HeaderParsingError::InvalidGame { .. }))
        ));
    }
}
