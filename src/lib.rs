//! # rediserde - A Redis RESP protocol serializer/deserializer for Rust
//! This library provides functionality to serialize and deserialize data
//! in the Redis RESP (Redis Serialization Protocol) format using serde.
//! Supports all RESP2 and RESP3 data types.
//!
//! ## RESP Data Types
//! For more information on RESP data types and their uses in this crate, see the documentation for
//! [`RespDataKind`] and refer to the [Redis RESP documentation](https://redis.io/docs/latest/protocol-spec/).
//!
//! ## Quick Start
//! Especially easy to use with [`serde::Serialize`] and [`serde::Deserialize`] derive macros.
//!
//! ```
//! use rediserde::{from_str, to_string};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq)]
//! struct Person {
//!    name: String,
//!    age: u32,
//! }
//!
//! let person = Person {
//!    name: "Alice".to_string(),
//!    age: 30,
//! };
//!
//! let serialized = to_string(&person).unwrap();
//! let deserialized: Person = from_str(&serialized).unwrap();
//! assert_eq!(deserialized, person);
//! ```
//!
//! But may also be used directly with [`to_string`], [`to_bytes`], [`from_str`], and [`from_bytes`].
//!
//! ```
//! use rediserde::{from_str, to_string};
//! let data = "*3\r\n:1\r\n:2\r\n:3\r\n";
//! let deserialized: Vec<u32> = from_str(&data).unwrap();
//! assert_eq!(deserialized, vec![1, 2, 3]);
//! let serialized = to_string(&deserialized).unwrap();
//! assert_eq!(serialized, data);
//! ```
//!

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
