# rediserde (Redis Serde)

[![Rust](https://github.com/ofersadan85/rediserde/actions/workflows/rust.yml/badge.svg)](https://github.com/ofersadan85/rediserde/actions/workflows/rust.yml)
[![Crates.io Version](https://img.shields.io/crates/v/rediserde)](https://crates.io/crates/rediserde)
[![docs.rs](https://img.shields.io/docsrs/rediserde)](https://docs.rs/rediserde)
[![GitHub License](https://img.shields.io/github/license/ofersadan85/rediserde)](https://github.com/ofersadan85/rediserde/blob/main/LICENSE)

A [Serde](https://serde.rs/) implementation for the [RESP](https://redis.io/docs/latest/develop/reference/protocol-spec/) (Redis Serialization Protocol) format, supporting both serialization and deserialization of Rust data structures.

## Features

- Serialize Rust types to RESP format
- Deserialize RESP data into Rust types
- Supports complex structs, enums, maps, arrays, options, and more
- Simple API: `to_string`, `to_bytes`, `from_str`, `from_bytes`
- **Full support of serde's derive macros**

## Installation

```bash
cargo add serde --features derive
cargo add rediserde
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
rediserde = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
```

> [!NOTE]
> `derive` feature is optional but recommended for ease of use (with `#[derive(Serialize, Deserialize)]`)

## Usage / Documentation

Use this crate like any other serde-compatible crate (like `serde_json` or `serde_yaml`):

```rust
    use rediserde::{from_str, to_string};
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }
    
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    let serialized = to_string(&person).unwrap();
    let deserialized: Person = from_str(&serialized).unwrap();
    assert_eq!(deserialized, person);
```

For a more complex example, see the [tests/structs.rs](tests/structs.rs) file.

- Full docs: [docs.rs/rediserde](https://docs.rs/rediserde).
- Repository: [github.com/ofersadan85/rediserde](https://github.com/ofersadan85/rediserde).
- Crate: [crates.io/crates/rediserde](https://crates.io/crates/rediserde).

For more details on how to use this crate, refer to the [Serde documentation](https://serde.rs/).

## RESP Format compatibility

This crate supports the full [RESP2 + RESP3 protocol specification](https://redis.io/docs/latest/develop/reference/protocol-spec/), including:

- Simple strings, errors, integers, bulk strings, arrays, maps, sets, nulls, booleans, floats (doubles), big numbers, verbatim strings, and more.

However, to match with Rust's (and serde's) types the mapping is roughly as follows:

| RESP Type       | Rust Type               |
|-----------------|-------------------------|
| Simple String   | `String`                |
| Error           | `String`                |
| Integer         | `u8`-`u64`, `i8`-`i64`  |
| Bulk String     | `String`                |
| Array           | `Vec<T>`                |
| Null            | `Option<T>` (`None`)    |
| Boolean         | `bool`                  |
| Double          | `f64`                   |
| Big Number      | `i64`                   |
| Verbatim String | `String`                |
| Map             | `HashMap<String, T>`    |
| Attribute       | `HashMap<String, T>`    |
| Set             | `Vec<T>`                |
| Push            | `Vec<T>`                |

However, since the mapping is not one-to-one, there are some important notes:

- RESP `Integer`s are deserializable to any Rust integer numeric type, assuming they fit within the range of the target type.
- RESP `Big Number`s are deserializable to all "smaller" Rust integer types assuming they fit within the range of the target type, but RESP `Integer` is at most `i64`, so a Rust `u64` (which might be bigger) will always be serialized as a `Big Number` while other numeric integer types will be serialized as RESP `Integer`s.
- RESP `Double`s (floating point numbers) are deserializable to both `f64` and `f32`, assuming they fit within the range of the target type.
- RESP `Map`s and `Attribute`s are both deserializable into structs, and `HashMap`s but structs and `HashMap`s are always serialized as RESP `Map`s.
- RESP `Array`s, `Set`s, and `Push`es are deserializable into any Rust sequence type (like `Vec`, `HashSet`, etc.) but Rust sequences are always serialized as RESP `Array`s.
- RESP's various string types (`Simple String`, `Simple Error`, `Bulk String`, `Bulk Error`, `Verbatim String`) are deserializable into a Rust `String`, but Rust `String`s are always serialized as RESP `Bulk String` (as this is the most common and versatile string type in RESP).
- Rust `String`s are guaranteed to be UTF-8 encoded, but RESP types are not, so deserializing will fail if the RESP data is not valid UTF-8. If you're unsure, deserialize to bytes (`Vec<u8>`) instead and handle the data manually.
- Rust's `u128` and `i128` are not supported by serde. If support is added there, we will follow and they will have to be serialized as RESP `Big Number`s.
- Rust does not support any primitive `Null` type, so creating a RESP `Null` is only possible in the context of an `Option<T>` where `T` is any type. The `None` variant will be serialized as RESP `Null` and vice versa.
- RESP concepts like a [Null Array](https://redis.io/docs/latest/develop/reference/protocol-spec/#null-arrays) or [Null String](https://redis.io/docs/latest/develop/reference/protocol-spec/#null-bulk-strings) are not easily representable in Rust, but reading such a value will not fail but yield an empty array or an empty string, respectively.
- While RESP supports maps and arrays with mixed types, Rust does not, so trying to get a Rust `HashMap<String, T>` or `Vec<T>` with mixed types will fail.
- Currently, only `String`s are supported as map keys (although RESP supports any type). This is planned to be extended in the future to support more types, but only as far as is reasonable for Rust, i.e. types that implement the `Hash` and `Eq` traits as required by `HashMap`.

## Notable Alternatives

- [serde-RESP](https://crates.io/crates/serde-resp): A similar crate that also implements RESP serialization and deserialization with serde, but with a different API and design choices. Unlike this crate, it does not directly support serde's derive macros on enums and structs, but has some handy macros for direct data manipulation. Appears to be unmaintained since February 2021. [GitHub](https://github.com/dedztbh/serde-RESP).
- [resp](https://crates.io/crates/resp): A crate RESP types but does not integrate with serde. It provides a low-level API for working with RESP data, but does not support serialization or deserialization of Rust types. Appears to be unmaintained since August 2022. [GitHub](https://github.com/iorust/resp).
- [stream_resp](https://crates.io/crates/stream_resp): A crate that provides a streaming API for RESP data, but does not support serde serialization or deserialization. Appears to be actively maintained. [GitHub](https://github.com/daydaydrunk/stream_resp).

To try and minimize dependencies and maximize flexibility in further development, this crate does not depend on any of the above crates, but rather implements RESP serialization and deserialization directly.

## Limitations and conditions

- This crate is in active development, and tries to choose the most sensible defaults, which might not perfectly match your use case. We plan to implement more serializers and deserializers if there's demand (or perhaps, enable crate features), so please [open an issue](https://github.com/ofersadan85/rediserde/issues) if you need a specific feature or behavior.
- Contributions are welcome!
- The RESP protocol is designed for Redis, so some features might not be applicable outside of that context. This crate is primarily intended for use with Redis or similar systems that use the RESP protocol.
- Performance has not been well tested, yet. If you want to help with that, please [open an issue](https://github.com/ofersadan85/rediserde/issues) or a PR.
- **This crate is provided "as is" without any warranties or guarantees. Use it at your own risk**.
