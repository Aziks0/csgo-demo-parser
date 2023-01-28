use bytes::Bytes;
use protobuf::CodedInputStream;

use crate::error::Result;

/// Delta-compressed user command.
///
/// See [Valve's community documentation][Valve Doc Usercmd] for more
/// information.
/// See also the [Source SDK] for how to read the compressed data.
///
/// [Valve Doc Usercmd]: https://developer.valvesoftware.com/wiki/Usercmd
/// [Source SDK]: https://github.com/ValveSoftware/source-sdk-2013/blob/master/mp/src/game/shared/usercmd.cpp#L199
#[derive(Debug)]
pub struct UserCommandCompressed {
    pub out_sequence: u32,
    pub data: Bytes,
}

impl UserCommandCompressed {
    pub(crate) fn try_new(reader: &mut CodedInputStream) -> Result<Self> {
        Ok(Self {
            out_sequence: reader.read_fixed32()?,
            data: reader.read_tokio_bytes()?,
        })
    }
}
