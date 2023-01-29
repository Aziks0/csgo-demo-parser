mod command;
mod console_command;
mod header;
mod user_command;

pub mod data_table;
pub mod packet;
pub mod string_table;

// Re-export them to be in `csgo_demo_parser::parser`
pub use command::{PacketContent, PacketHeader};
pub use header::DemoHeader;
pub use user_command::UserCommandCompressed;

use getset::Getters;
use protobuf::CodedInputStream;
use std::io;
use tracing::{instrument, trace};

use crate::error::Result;
use crate::parser::command::Command;
use crate::parser::console_command::ConsoleCommand;
use crate::parser::data_table::DataTables;
use crate::parser::packet::Packet;
use crate::parser::string_table::StringTables;

#[derive(Getters, Debug)]
#[getset(get = "pub")]
pub struct DemoParser<'a> {
    #[getset(skip)]
    reader: CodedInputStream<'a>,
    header: DemoHeader,
}

impl<'a> DemoParser<'a> {
    /// Create a new [`DemoParser`] based on a [`Read`]. The header of
    /// the demo file will get parsed.
    ///
    /// Note: the underlying reader will be buffered. If [`Read`] is
    /// already buffered, consider using [`from_buf_read`](Self::from_buf_read) instead.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered by this function will result in an error
    /// being returned.
    ///
    /// If the parsed header of the demo file contains invalid data, an error
    /// will be returned. See [`HeaderParsingError`](crate::error::HeaderParsingError).
    ///
    /// # Examples
    ///
    /// [`File`]s implement [`Read`]:
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use csgo_demo_parser::DemoParser;
    ///
    /// # fn main() -> csgo_demo_parser::error::Result<()> {
    /// let mut demo_file = File::open("foo.dem")?;
    /// let demo_parser = DemoParser::try_new(&mut demo_file)?;
    /// println!("Map played: {}", demo_parser.header().map_name());
    /// #
    /// #   Ok(())
    /// # }
    /// ```
    ///
    /// [`Read`]: std::io::Read
    /// [`File`]: std::fs::File
    ///
    #[instrument(level = "trace", skip(read))]
    pub fn try_new(read: &'a mut dyn io::Read) -> Result<Self> {
        let mut reader = CodedInputStream::new(read);
        let header = DemoHeader::try_new(&mut reader)?;
        trace!(?header);

        Ok(Self { header, reader })
    }

    /// Create a new [`DemoParser`] based on a [`BufRead`](std::io::BufRead).
    /// The header of the demo file will get parsed.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered by this function will result in an error
    /// being returned.
    ///
    /// If the parsed header of the demo file contains invalid data, an error
    /// will be returned. See [`HeaderParsingError`](crate::error::HeaderParsingError).
    ///
    #[instrument(level = "trace", skip(buf_read))]
    pub fn from_buf_read(buf_read: &'a mut dyn io::BufRead) -> Result<Self> {
        let mut reader = CodedInputStream::from_buf_read(buf_read);
        let header = DemoHeader::try_new(&mut reader)?;
        trace!(?header);

        Ok(Self { header, reader })
    }

    /// Iterator-like method to iterate over every packets. When there are no
    /// more packet to parse, it will return `Ok(None)` indefinitely.
    ///
    /// # Errors
    ///
    /// Any I/O error encountered by this function will result in an error
    /// being returned.
    ///
    /// If the parser encounters any invalid packet, or more generally any
    /// invalid data, an error will be returned.
    ///
    /// See [`Error`](crate::error::Error) for a list of possible errors.
    ///
    /// # Exemples
    ///
    /// ```no_run
    /// use std::fs::File;
    /// use csgo_demo_parser::DemoParser;
    ///
    /// # fn main() -> csgo_demo_parser::error::Result<()> {
    /// let mut demo_file = File::open("foo.dem")?;
    /// let mut demo_parser = DemoParser::try_new(&mut demo_file)?;
    ///
    /// // Iterate through every packets until the end of the demo
    /// while let Some((packet_header, packet)) = demo_parser.parse_next_packet()? {
    ///     println!("{:#?}", packet_header);
    ///     println!("{:#?}", packet);
    /// }
    /// #   Ok(())
    /// # }
    /// ```
    #[instrument(level = "trace", skip(self))]
    pub fn parse_next_packet(&mut self) -> Result<Option<(PacketHeader, PacketContent)>> {
        if self.reader.eof()? {
            return Ok(None);
        }

        let header = PacketHeader::try_new(&mut self.reader)?;
        trace!(?header);

        Ok(match header.command {
            Command::Stop => Some((header, PacketContent::Stop)),
            Command::SyncTick => Some((header, PacketContent::SyncTick)),
            Command::ConsoleCommand => {
                let console_command = ConsoleCommand::try_new(&mut self.reader)?;
                trace!(?console_command);
                Some((
                    header,
                    PacketContent::ConsoleCommand(console_command.command),
                ))
            }
            Command::UserCommand => {
                let user_command = UserCommandCompressed::try_new(&mut self.reader)?;
                trace!(?user_command);
                Some((header, PacketContent::UserCommand(user_command)))
            }
            Command::Packet | Command::Signon => {
                let packet = Packet::try_new(&mut self.reader)?;
                Some((header, PacketContent::Packet(packet.messages)))
            }
            Command::StringTables => {
                let string_tables = StringTables::try_new(&mut self.reader)?;
                Some((header, PacketContent::StringTables(string_tables.tables)))
            }
            Command::DataTables => {
                let data_tables = DataTables::try_new(&mut self.reader)?;
                Some((header, PacketContent::DataTables(data_tables)))
            }
            Command::CustomData => unimplemented!("custom data found"),
        })
    }
}
