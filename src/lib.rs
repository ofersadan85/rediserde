mod de;
mod error;
mod resp;
mod ser;

pub use de::{Deserializer, from_bytes, from_str};
pub use error::{Error, Result};
pub use resp::RespDataKind;
pub use ser::{Serializer, to_bytes, to_string, to_utf8_lossy};

pub const CRLF: &[u8] = b"\r\n";
pub const CRLF_STR: &str = "\r\n";
