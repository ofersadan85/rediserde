/// An enumeration representing the different kinds of RESP data types.
///
/// Mainly used internally to handle serialization and deserialization
/// by the first character of the RESP data type prefix.
///
/// Each variant corresponds to a specific RESP data type,
/// and may be serialized or deserialized to specific Rust types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RespDataKind {
    /// Represents a RESP [Simple String](https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-strings)
    ///
    /// Prefix: `+` | for example, `+OK\r\n`
    ///
    /// Serialized as [`String`], so required to be UTF-8 encoded (even though RESP does not enforce this).
    ///
    /// [`String`]s *do not* deserialize to this type, but instead to [`RespDataKind::BulkString`].
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// let s = "+OK\r\n";
    /// let rust_string: String = from_str(s).unwrap();
    /// assert_eq!(rust_string, "OK");
    /// let bulk_string = to_string(&rust_string).unwrap();
    /// assert_ne!(bulk_string, s, "Serialized as BulkString, not SimpleString");
    /// assert_eq!(bulk_string, "$2\r\nOK\r\n", "Serialized as BulkString, not SimpleString");
    /// ```
    SimpleString,
    /// Represents a RESP [Simple Error](https://redis.io/docs/latest/develop/reference/protocol-spec/#simple-errors)
    ///
    /// Prefix: `-` | for example, `-ERR unknown command\r\n`
    ///
    /// Serialized as [`String`], so required to be UTF-8 encoded (even though RESP does not enforce this).
    ///
    /// [`String`]s *do not* deserialize to this type, but instead to [`RespDataKind::BulkString`].
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// let s = "-ERR unknown command\r\n";
    /// let rust_string: String = from_str(s).unwrap();
    /// assert_eq!(rust_string, "ERR unknown command");
    /// let bulk_string = to_string(&rust_string).unwrap();
    /// assert_ne!(bulk_string, s, "Serialized as BulkString, not SimpleError");
    /// assert_eq!(bulk_string, "$19\r\nERR unknown command\r\n", "Serialized as BulkString, not SimpleError");
    /// ```
    SimpleError,
    /// Represents a RESP [Integer](https://redis.io/docs/latest/develop/reference/protocol-spec/#integers)
    ///
    /// Prefix: `:` | for example, `:42\r\n` or `:+42\r\n` for positive and `:-42\r\n` for negative
    ///
    /// Serialized and deserialized as [`i64`], but can be used with any integer type that
    /// implements `Into<i64>`. If the RESP integer is outside the range
    /// of the target type, it will result in an error.
    ///
    /// [`u64`] and [`usize`] are can be constructed from RESP integers, but the reverse is not true
    /// since RESP integers are at most 64 bits (including negative values). Serializing
    /// [`u64`] or [`usize`] to RESP will use [`RespDataKind::BigNumber`] instead.
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// let s = ":42\r\n";
    /// let int8: i8 = from_str(s).unwrap();
    /// assert_eq!(int8, 42);
    /// let int16: i64 = from_str(s).unwrap();
    /// assert_eq!(int16, 42);
    /// let u_int8: u8 = from_str(s).unwrap();
    /// assert_eq!(u_int8, 42);
    /// // Can also become a `u64` or `usize`, but the reverse is not true
    /// let u_int64: u64 = from_str(s).unwrap();
    /// assert_eq!(u_int64, 42);
    /// let u64_str = to_string(&u_int64).unwrap();
    /// assert_ne!(u64_str, s, "Serialized as BigNumber, not Integer");
    /// assert_eq!(u64_str, "(42\r\n", "Serialized as BigNumber, not Integer");
    /// ```
    Integer,
    /// Represents a RESP [Bulk String](https://redis.io/docs/latest/develop/reference/protocol-spec/#bulk-strings)
    ///
    /// Prefix: `$` | for example, `$6\r\nfoobar\r\n` (where `6` is the length of the string)
    ///
    /// Serialized as [`String`], so required to be UTF-8 encoded (even though RESP does not enforce this).
    ///
    /// Unlike the other string types, this variant will always be the result of deserializing
    /// a Rust [`String`].
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// let s = "$4\r\nTEST\r\n";
    /// let rust_string: String = from_str(s).unwrap();
    /// assert_eq!(rust_string, "TEST");
    /// let bulk_string = to_string(&rust_string).unwrap();
    /// assert_eq!(bulk_string, s);
    /// ```
    BulkString,
    /// Represents a RESP [Array](https://redis.io/docs/latest/develop/reference/protocol-spec/#arrays)
    ///
    /// Prefix: `*` | for example, `*3\r\n<item1><item2><item3>` (where `3` is the number of items)
    ///
    /// Equivalent to a Rust [`Vec<T>`] where `T` is the type of the items in the array. While RESP
    /// arrays support mixed types, Rust does not, so all items must be of the same type.
    ///
    /// Can be serialized and deserialized to and from any collection type that implements
    /// [`serde::Serialize`] and [`serde::Deserialize`] respectively, such as
    /// [`Vec<T>`], [`std::collections::HashSet<T>`], or [`std::collections::BTreeSet<T>`].
    ///
    /// Although RESP has a `Set` type, Rust [`std::collections::HashSet<T>`] and
    /// [`std::collections::BTreeSet<T>`] are always serialized as RESP arrays,
    /// but RESP sets can be deserialized to any collection without issue.
    ///
    /// ```
    /// # use std::collections::BTreeSet;
    /// # use rediserde::{from_str, to_string};
    /// let resp_array_str = "*2\r\n:1\r\n:2\r\n";
    /// let resp_set_str = "~2\r\n:1\r\n:2\r\n";
    /// let vec1: Vec<i64> = from_str(resp_array_str).unwrap();
    /// let vec2: Vec<i64> = from_str(resp_set_str).unwrap();
    /// assert_eq!(vec1, vec![1, 2]);
    /// assert_eq!(vec2, vec![1, 2]);
    /// let array_string = to_string(&vec1).unwrap();
    /// assert_eq!(array_string, resp_array_str);
    /// // HashSet can be used, but might not preserve order, so we use BTreeSet for example
    /// let my_set = BTreeSet::from_iter(vec1);
    /// let set_string = to_string(&my_set).unwrap();
    /// assert_eq!(set_string, resp_array_str, "Serialized as RESP Array, not Set");
    /// assert_ne!(set_string, resp_set_str, "Serialized as RESP Array, not Set");
    /// ```
    Array,
    /// Represents a RESP [Null](https://redis.io/docs/latest/develop/reference/protocol-spec/#nulls)
    ///
    /// Prefix: `_` | for example, `_\r\n`
    ///
    /// Rust does not have a direct equivalent for this type, but it can be used
    /// to represent a missing value in a collection or as an [`Option::None`] of an [`Option<T>`].
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// // This is an array of 3 items, with the second item being null
    /// let array_str = "*3\r\n:1\r\n_\r\n:3\r\n";
    /// let vec: Vec<Option<i64>> = from_str(array_str).unwrap();
    /// assert_eq!(vec, vec![Some(1), None, Some(3)]);
    /// ```
    Null,
    /// Represents a RESP [Boolean](https://redis.io/docs/latest/develop/reference/protocol-spec/#booleans)
    ///
    /// Prefix: `#` | for example, `#t\r\n` (`true`) or `#f\r\n` (`false`)
    ///
    /// Serialized as a Rust [`bool`]
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// let true_str = "#t\r\n";
    /// let false_str = "#f\r\n";
    /// let true_bool: bool = from_str(true_str).unwrap();
    /// assert!(true_bool);
    /// let false_bool: bool = from_str(false_str).unwrap();
    /// assert!(!false_bool);
    Boolean,
    /// Represents a RESP [Float](https://redis.io/docs/latest/develop/reference/protocol-spec/#floats)
    ///
    /// Prefix: `,` | for example, `,3.1\r\n`, `,+3.1\r\n` for positive and `,-3.1\r\n` for negative
    /// Serialized and deserialized as a both [`f32`] and [`f64`]
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// let float_str = ",3.1\r\n";
    /// let float_f32: f32 = from_str(float_str).unwrap();
    /// assert_eq!(float_f32, 3.1);
    Float,
    /// Represents a RESP [Big Number](https://redis.io/docs/latest/develop/reference/protocol-spec/#big-numbers)
    ///
    /// Prefix: `(` | for example, `(12345678901234567890\r\n`
    ///
    /// The only numeric Rust types that can be serialized to this type are
    /// [`u64`] and [`usize`], since the max range of normal RESP integers is 64 bits
    /// (including negative values). These types automatically convert to this RESP type,
    /// even if they are "smaller" than 64 bits to make the logic consistent.
    /// This behavior may change in the future.
    ///
    /// Currently, [`u128`] and [`i128`] are not supported
    /// by serde, so they cannot be used with this crate (at the moment).
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// let num64 = 42_u64;
    /// let num64_str = to_string(&num64).unwrap();
    /// assert_eq!(num64_str, "(42\r\n");
    /// let num64_deserialized: u64 = from_str(&num64_str).unwrap();
    /// // but within range, this can also be coerced to a `u8` for example
    /// let num8: u8 = from_str(&num64_str).unwrap();
    /// ```
    BigNumber,
    /// Represents a RESP [Bulk Error](https://redis.io/docs/latest/develop/reference/protocol-spec/#bulk-errors)
    ///
    /// Prefix: `!` | for example, `!5\r\nError\r\n` (where `5` is the length of the error message)
    ///
    /// Serialized as [`String`], so required to be UTF-8 encoded (even though RESP does not enforce this).
    ///
    /// Similar to [`RespDataKind::BulkString`], but semantically used for errors. Can be deserialized
    /// to a Rust [`String`], but is not used for serialization (as RESP does not have a direct
    /// equivalent for this type). See the documentation for [`RespDataKind::SimpleString`]
    /// and [`RespDataKind::BulkString`] for more details.
    BulkError,
    /// Represents a RESP [Verbatim String](https://redis.io/docs/latest/develop/reference/protocol-spec/#verbatim-strings)
    ///
    /// Prefix: `=` | Structured as `=<length>\r\n<encoding>:<data>\r\n` where `<length>` is the
    /// length of the data and `<encoding>` is exactly 3 bytes long, representing the encoding type.
    ///
    /// Serialized as [`String`], so required to be UTF-8 encoded (even though RESP does not enforce this).
    ///
    /// Currently, this will only serialize and deserialize to UTF-8 encoded strings, but we plan
    /// to support other encodings in the future, or fallback to bytes ([`Vec<u8>`]) if the encoding
    /// is not supported. See the documentation for [`RespDataKind::SimpleString`]
    /// and [`RespDataKind::BulkString`] for more details.
    VerbatimString,
    /// Represents a RESP [Map](https://redis.io/docs/latest/develop/reference/protocol-spec/#maps)
    ///
    /// Prefix: `%` | for example, `%1\r\n$3\r\nkey\r\n$5\r\nvalue\r\n`
    /// (where `1` is the number of key-value pairs)
    ///
    /// Can be serialized and deserialized to and from a Rust [`std::collections::HashMap<String, T>`],
    /// [`std::collections::BTreeMap<String, T>`], or even a struct with named fields that implements
    /// [`serde::Serialize`] and/or [`serde::Deserialize`]. Note that for Rust map types, values must
    /// be the same type, even though RESP maps can have mixed types.
    ///
    /// Currently, the keys are always serialized as [`String`], so they must be UTF-8 encoded. However,
    /// RESP does allows for other key types, so we may add support for those in the future.
    ///
    /// A derive example for structs:
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// # use serde::{Serialize, Deserialize};
    /// #[derive(Debug, Serialize, Deserialize, PartialEq)]
    /// struct Person {
    ///     name: String,
    ///     age: u32,
    /// }
    ///
    /// let person = Person {
    ///     name: "Alice".to_string(),
    ///     age: 30,
    /// };
    /// let serialized = to_string(&person).unwrap();
    /// let deserialized: Person = from_str(&serialized).unwrap();
    /// assert_eq!(deserialized, person);
    /// ```
    ///
    /// And an arbitrary map example:
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// # use std::collections::{HashMap, BTreeMap};
    ///
    /// let mut map = HashMap::new();
    /// map.insert("first_name".to_string(), "Alice".to_string());
    /// map.insert("last_name".to_string(), "Smith".to_string());
    ///
    /// let serialized = to_string(&map).unwrap();
    /// assert_eq!(serialized, "%2\r\n$10\r\nfirst_name\r\n$5\r\nAlice\r\n$9\r\nlast_name\r\n$5\r\nSmith\r\n");
    /// let deserialized_hashmap: HashMap<String, String> = from_str(&serialized).unwrap();
    /// assert_eq!(deserialized_hashmap, map);
    /// let deserialized_bt: BTreeMap<String, String> = from_str(&serialized).unwrap();
    /// assert_eq!(deserialized_bt.len(), 2);
    /// ```
    Map,
    /// Represents a RESP [Attributes](https://redis.io/docs/latest/develop/reference/protocol-spec/#attributes)
    ///
    /// Prefix: `|` | for example, `|1\r\n$3\r\nkey\r\n$5\r\nvalue\r\n`
    /// (where `1` is the number of key-value pairs)
    ///
    /// Identical to [`RespDataKind::Map`], but used for attributes semantically.
    ///
    /// Since there is no direct equivalent in Rust, it can be used for deserialization but not
    /// for serialization. See the documentation for [`RespDataKind::Map`] for more details.
    ///
    /// ```
    /// # use rediserde::{from_str, to_string};
    /// # use std::collections::BTreeMap;
    /// let attr_str = "|1\r\n$3\r\nkey\r\n$5\r\nvalue\r\n";
    /// // We could use a `HashMap` but `BTreeMap` preserves order and is easier to test
    /// let mut attributes: BTreeMap<String, String> = from_str(attr_str).unwrap();
    /// assert_eq!(attributes.len(), 1);
    /// assert_eq!(attributes.get("key").unwrap(), "value");
    /// ```
    Attributes,
    /// Represents a RESP [Set](https://redis.io/docs/latest/develop/reference/protocol-spec/#sets)
    ///
    /// Prefix: `~` | for example, `~2\r\n:1\r\n:2\r\n` (where `2` is the number of items)
    ///
    /// Used only for deserialization and not for serialization.
    ///
    /// Although RESP has a `Set` type and Rust has [`std::collections::HashSet<T>`] and
    /// [`std::collections::BTreeSet<T>`], these are always serialized as RESP arrays,
    /// since serde can only represent a "collection" without distinguishing between them.
    ///
    /// See the documentation for [`RespDataKind::Array`] for more details and examples.
    Set,
    /// Represents a RESP [Push](https://redis.io/docs/latest/develop/reference/protocol-spec/#pushes)
    ///
    /// Prefix: `>` | for example, `>2\r\n:1\r\n:2\r\n` (where `2` is the number of items)
    ///
    /// Identical to [`RespDataKind::Array`], but used for pushes semantically.
    ///
    /// Used only for deserialization and not for serialization.
    ///
    /// See the documentation for [`RespDataKind::Array`] for more details and examples.
    Push,
}

impl RespDataKind {
    pub(crate) fn to_prefix_char(self) -> char {
        match self {
            Self::SimpleString => '+',
            Self::SimpleError => '-',
            Self::Integer => ':',
            Self::BulkString => '$',
            Self::Array => '*',
            Self::Null => '_',
            Self::Boolean => '#',
            Self::Float => ',',
            Self::BigNumber => '(',
            Self::BulkError => '!',
            Self::VerbatimString => '=',
            Self::Map => '%',
            Self::Attributes => '|',
            Self::Set => '~',
            Self::Push => '>',
        }
    }

    pub(crate) fn to_prefix_bytes(self) -> u8 {
        u8::try_from(self.to_prefix_char()).expect("All prefixes are known ASCII characters")
    }

    fn from_prefix_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::SimpleString),
            '-' => Some(Self::SimpleError),
            ':' => Some(Self::Integer),
            '$' => Some(Self::BulkString),
            '*' => Some(Self::Array),
            '_' => Some(Self::Null),
            '#' => Some(Self::Boolean),
            ',' => Some(Self::Float),
            '(' => Some(Self::BigNumber),
            '!' => Some(Self::BulkError),
            '=' => Some(Self::VerbatimString),
            '%' => Some(Self::Map),
            '|' => Some(Self::Attributes),
            '~' => Some(Self::Set),
            '>' => Some(Self::Push),
            _ => None,
        }
    }

    fn from_prefix_bytes(b: u8) -> Option<Self> {
        Self::from_prefix_char(char::from(b))
    }
}

impl From<RespDataKind> for u8 {
    fn from(kind: RespDataKind) -> Self {
        kind.to_prefix_bytes()
    }
}

impl From<RespDataKind> for char {
    fn from(kind: RespDataKind) -> Self {
        kind.to_prefix_char()
    }
}

impl TryFrom<char> for RespDataKind {
    type Error = ();

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        Self::from_prefix_char(value).ok_or(())
    }
}

impl TryFrom<u8> for RespDataKind {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        Self::from_prefix_bytes(value).ok_or(())
    }
}

impl std::fmt::Display for RespDataKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_prefix_char())
    }
}
