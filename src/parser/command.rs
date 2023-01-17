use getset::Getters;
use protobuf::CodedInputStream;

use crate::error::{Error, Result};

#[derive(Debug)]
pub(crate) enum Command {
    Signon = 1,
    Packet,
    SyncTick,
    ConsoleCommand,
    UserCommand,
    DataTables,
    Stop,
    CustomData,
    StringTables,
}

impl TryFrom<u8> for Command {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Signon),
            2 => Ok(Self::Packet),
            3 => Ok(Self::SyncTick),
            4 => Ok(Self::ConsoleCommand),
            5 => Ok(Self::UserCommand),
            6 => Ok(Self::DataTables),
            7 => Ok(Self::Stop),
            8 => Ok(Self::CustomData),
            9 => Ok(Self::StringTables),
            _ => Err(Error::UnknownPacketCommand(value)),
        }
    }
}

/// Packet header.
#[derive(Getters, Debug)]
#[getset(get = "pub")]
pub struct PacketHeader {
    /// The [`Command`] for this packet.
    #[getset(skip)]
    pub(crate) command: Command,
    /// The tick at which the packet was emitted.
    tick: i32,
    /// The player slot from which the packet was emitted.
    player_slot: u8,
}

impl PacketHeader {
    pub(crate) fn try_new(reader: &mut CodedInputStream) -> Result<Self> {
        Ok(Self {
            command: reader.read_raw_byte()?.try_into()?,
            tick: reader.read_sfixed32()?,
            player_slot: reader.read_raw_byte()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    use protobuf::CodedInputStream;

    #[test]
    fn valid_packet_header() {
        let mut bytes: &[u8] = &[1, 60, 0, 0, 0, 0];
        let mut buf = CodedInputStream::new(&mut bytes);
        let packet_header = PacketHeader::try_new(&mut buf).unwrap();
        assert!(matches!(packet_header.command, Command::Signon));
        assert_eq!(packet_header.tick, 60);
        assert_eq!(packet_header.player_slot, 0);
    }

    #[test]
    fn error_on_unknown_command() {
        let mut bytes: &[u8] = &[56, 60, 0, 0, 0, 0];
        let mut buf = CodedInputStream::new(&mut bytes);
        let res = PacketHeader::try_new(&mut buf);
        assert!(matches!(res, Err(Error::UnknownPacketCommand(56))));
    }
}
