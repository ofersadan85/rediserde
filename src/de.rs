#![allow(clippy::missing_errors_doc)]

use crate::{CRLF, CRLF_STR, Error, RespDataKind, Result};
use serde::de::IntoDeserializer;

const VALID_NUMERIC_CHARS: &[u8] = b"0123456789+-.eE";

pub struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    #[must_use]
    pub const fn new(input: &'de [u8]) -> Self {
        Self { input }
    }

    fn next_byte(&mut self) -> Result<u8> {
        if let Some(&byte) = self.input.first() {
            self.input = &self.input[1..];
            Ok(byte)
        } else {
            Err(Error::UnexpectedEnd)
        }
    }

    /// Inspect the current input for debugging purposes.
    #[allow(dead_code)]
    fn inspect(&self) {
        let input_lossy = String::from_utf8_lossy(self.input);
        dbg!(input_lossy);
    }

    /// Consumes the next byte and checks that it matches the expected byte.
    fn expect_byte(&mut self, expected: u8) -> Result<()> {
        let first = self.next_byte()?;
        if first == expected {
            Ok(())
        } else {
            Err(Error::UnexpectedByte {
                expected: char::from(expected).to_string(),
                found: char::from(first),
            })
        }
    }

    /// Consumes the next bytes to expect CRLF.
    fn expect_crlf(&mut self) -> Result<()> {
        if self.input.starts_with(CRLF) {
            self.input = &self.input[CRLF.len()..];
            Ok(())
        } else if self.input.is_empty() {
            Err(Error::UnexpectedEnd)
        } else {
            Err(Error::UnexpectedByte {
                expected: CRLF_STR.to_string(),
                found: char::from(self.input[0]),
            })
        }
    }

    /// Expects and consumes a numeric value
    fn expect_length(&mut self) -> Result<usize> {
        let first_non_numeric = self
            .input
            .iter()
            .position(|&b| !b.is_ascii_digit())
            .ok_or(Error::ExpectedLength)?;
        let length_str = String::from_utf8(self.input[..first_non_numeric].to_vec())
            .map_err(|_| Error::ExpectedLength)?;
        self.input = &self.input[first_non_numeric..];
        let length = length_str
            .parse::<usize>()
            .map_err(|_| Error::ExpectedLength)?;
        Ok(length)
    }

    fn parse_string(&mut self) -> Result<String> {
        let first = self.next_byte()?;
        let kind = RespDataKind::try_from(first).map_err(|()| Error::UnrecognizedStart)?;
        let result = match kind {
            RespDataKind::SimpleString
            | RespDataKind::SimpleError
            | RespDataKind::Integer
            | RespDataKind::BigNumber
            | RespDataKind::Float => self.parse_simple_string(),
            RespDataKind::BulkString | RespDataKind::BulkError | RespDataKind::VerbatimString => {
                self.parse_bulk_string()
            }
            _ => Err(Error::UnexpectedByte {
                expected: "A string or number prefix".to_string(),
                found: char::from(first),
            }),
        }?;
        Ok(result)
    }

    fn parse_simple_string(&mut self) -> Result<String> {
        let crlf_index = self.input.windows(2).position(|w| w == CRLF);
        let result = if let Some(index) = crlf_index {
            let result = &self.input[..index];
            self.input = &self.input[index..];
            result
        } else {
            return Err(Error::UnexpectedEnd);
        };
        if result.is_empty() {
            return Err(Error::UnexpectedEnd);
        }
        self.expect_crlf()?;
        Ok(String::from_utf8(result.to_vec())?)
    }

    fn parse_bulk_string(&mut self) -> Result<String> {
        if self.input.starts_with(b"-1\r\n") {
            self.input = &self.input[4..]; // Skip -1\r\n
            return Ok(String::new()); // Null string
        }
        let length = self.expect_length()?;
        self.expect_crlf()?;
        let data = &self.input[..length];
        self.input = &self.input[length..];
        self.expect_crlf()?;
        Ok(String::from_utf8(data.to_vec())?)
    }

    /// Parse an number from the RESP format.
    /// The integer format is: :<value>\r\n
    /// The float format is: ,[<+|->]<integral>[.<fractional>][<E|e>[sign]<exponent>]\r\n
    /// The big number format is: ([+|-]<number>\r\n
    fn parse_number<N>(&mut self) -> Result<N>
    where
        N: std::str::FromStr + std::fmt::Debug + Copy,
    {
        let first = self.next_byte()?;
        let kind = RespDataKind::try_from(first).map_err(|()| Error::UnrecognizedStart)?;
        if !matches!(
            kind,
            RespDataKind::Integer | RespDataKind::Float | RespDataKind::BigNumber
        ) {
            return Err(Error::UnexpectedByte {
                expected: "An integer (:), float (,), or big number (() prefix".to_string(),
                found: char::from(first),
            });
        }
        let non_numeric_index = self
            .input
            .iter()
            .position(|b| !VALID_NUMERIC_CHARS.contains(b))
            .ok_or(Error::UnexpectedEnd)?;
        let value_str = String::from_utf8(self.input[..non_numeric_index].to_vec())?;
        self.input = &self.input[non_numeric_index..];
        let value = value_str.parse::<N>().map_err(|_| Error::UnexpectedByte {
            expected: "A valid integer string".to_string(),
            found: value_str.chars().next().unwrap_or_default(),
        })?;
        self.expect_crlf()?;
        Ok(value)
    }
}

pub fn from_bytes<'de, T>(bytes: &'de [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut deserializer = Deserializer::new(bytes);
    T::deserialize(&mut deserializer)
}

pub fn from_str<'de, T>(s: &'de str) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    from_bytes(s.as_bytes())
}

impl<'de> serde::de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let first = self.input.first().ok_or(Error::UnexpectedEnd)?;
        let kind = RespDataKind::try_from(*first).map_err(|()| Error::UnrecognizedStart)?;
        match kind {
            RespDataKind::SimpleString
            | RespDataKind::SimpleError
            | RespDataKind::BulkString
            | RespDataKind::BulkError
            | RespDataKind::VerbatimString => self.deserialize_string(visitor),
            RespDataKind::Integer => self.deserialize_i64(visitor),
            RespDataKind::Array | RespDataKind::Set | RespDataKind::Push => {
                self.deserialize_seq(visitor)
            }
            RespDataKind::Null => self.deserialize_unit(visitor),
            RespDataKind::Boolean => self.deserialize_bool(visitor),
            RespDataKind::Float => self.deserialize_f64(visitor),
            RespDataKind::BigNumber => self.deserialize_i128(visitor),
            RespDataKind::Map | RespDataKind::Attributes => self.deserialize_map(visitor),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.expect_byte(RespDataKind::Boolean.to_prefix_bytes())?;
        let value = match self.next_byte()? {
            b't' => true,
            b'f' => false,
            b => {
                return Err(Error::UnexpectedByte {
                    expected: "One of `t` or `f`".to_string(),
                    found: char::from(b),
                });
            }
        };
        self.expect_crlf()?;
        visitor.visit_bool(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i8(self.parse_number::<i8>()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i16(self.parse_number::<i16>()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i32(self.parse_number::<i32>()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_i64(self.parse_number::<i64>()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u8(self.parse_number::<u8>()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u16(self.parse_number::<u16>()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u32(self.parse_number::<u32>()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_u64(self.parse_number::<u64>()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f32(self.parse_number::<f32>()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number::<f64>()?)
    }

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let s = self.parse_string()?;
        if s.len() > 1 {
            return Err(Error::DeserializeError(
                "Expected a single character string".to_string(),
            ));
        }
        let c = s.chars().next().ok_or_else(|| {
            Error::DeserializeError("String is empty, expected a single character".to_string())
        })?;
        visitor.visit_char(c)
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let s = self.parse_string()?;
        visitor.visit_string(s)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bytes(self.input)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_bytes(self.input.as_ref())
    }

    /// The following is taken from the JSON documentation, and applies to RESP as well:
    ///
    /// An absent optional is represented as the JSON `null` and a present
    /// optional is represented as just the contained value.
    ///
    /// As commented in `Serializer` implementation, this is a lossy
    /// representation. For example the values `Some(())` and `None` both
    /// serialize as just `null`. Unfortunately this is typically what people
    /// expect when working with JSON. Other formats are encouraged to behave
    /// more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        if self
            .input
            .starts_with(&[RespDataKind::Null.to_prefix_bytes()])
        {
            self.deserialize_unit(visitor)
        } else {
            visitor.visit_some(self)
        }
    }

    /// In Serde, unit means an anonymous value containing no data.
    /// In RESP, this is a Null represented as `_` followed by CRLF.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.expect_byte(RespDataKind::Null.to_prefix_bytes())?;
        self.expect_crlf()?;
        visitor.visit_unit()
    }

    /// Unit struct means a named value containing no data.
    /// In RESP, this is a Null represented as `_` followed by CRLF.
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    /// As is done here, serializers are encouraged to treat newtype structs as
    /// insignificant wrappers around the data they contain. That means not
    /// parsing anything other than the contained value.
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    /// Deserialization of compound types like sequences and maps happens by
    /// passing the visitor an "Access" object that gives it the ability to
    /// iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let first = self.input.first().ok_or(Error::UnexpectedEnd)?;
        let kind = RespDataKind::try_from(*first).map_err(|()| Error::UnrecognizedStart)?;
        if !matches!(kind, RespDataKind::Array | RespDataKind::Set | RespDataKind::Push) {
            return Err(Error::UnexpectedByte {
                expected: "An array, set, or push prefix".to_string(),
                found: char::from(*first),
            });
        }
        self.expect_byte(*first)?;
        let length = self.expect_length()?;
        self.expect_crlf()?;
        // We need to create a new visitor that can handle the sequence
        let seq_visitor = LengthSeqVisitor::new(self, length);
        visitor.visit_seq(seq_visitor)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let first = self.input.first().ok_or(Error::UnexpectedEnd)?;
        let kind = RespDataKind::try_from(*first).map_err(|()| Error::UnrecognizedStart)?;
        if !matches!(kind, RespDataKind::Map | RespDataKind::Attributes) {
            return Err(Error::UnexpectedByte {
                expected: "A map or attributes prefix".to_string(),
                found: char::from(*first),
            });
        }
        self.expect_byte(*first)?;
        let length = self.expect_length()?;
        self.expect_crlf()?;

        let seq_visitor = LengthSeqVisitor::new(self, length);
        visitor.visit_map(seq_visitor)
    }

    // Structs look just like maps in RESP.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        let first = self.input.first().ok_or(Error::UnexpectedEnd)?; // Peek the first byte without consuming it
        let kind = RespDataKind::try_from(*first).map_err(|()| Error::UnrecognizedStart)?;
        match kind {
            RespDataKind::SimpleString
            | RespDataKind::SimpleError
            | RespDataKind::BulkString
            | RespDataKind::BulkError
            | RespDataKind::VerbatimString => {
                // Visit a unit variant.
                let s = self.parse_string()?;
                visitor.visit_enum(s.as_str().into_deserializer())
            }
            RespDataKind::Map | RespDataKind::Attributes => {
                visitor.visit_enum(EnumDeserializer::new(self))
            }
            _ => Err(Error::UnexpectedByte {
                expected: "A string or map prefix".to_string(),
                found: char::from(*first),
            }),
        }
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct LengthSeqVisitor<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    length: usize,
    current: usize,
}

impl<'a, 'de> LengthSeqVisitor<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, length: usize) -> Self {
        Self {
            de,
            length,
            current: 0,
        }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de> serde::de::SeqAccess<'de> for LengthSeqVisitor<'_, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        // Check if we have reached the end of the sequence.
        if self.current >= self.length {
            return Ok(None);
        }
        self.current += 1;

        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de> serde::de::MapAccess<'de> for LengthSeqVisitor<'_, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        // Check if we have reached the end of the sequence.
        if self.current >= self.length {
            return Ok(None);
        }
        self.current += 1;

        // Deserialize a map key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct EnumDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> EnumDeserializer<'a, 'de> {
    const fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de> serde::de::EnumAccess<'de> for EnumDeserializer<'_, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let first = self.de.input.first().ok_or(Error::UnexpectedEnd)?;
        let kind = RespDataKind::try_from(*first).map_err(|()| Error::UnrecognizedStart)?;
        if !matches!(kind, RespDataKind::Map | RespDataKind::Attributes) {
            return Err(Error::UnexpectedByte {
                expected: "A map or attributes prefix".to_string(),
                found: char::from(*first),
            });
        }
        self.de.expect_byte(*first)?;
        let length = self.de.expect_length()?;
        if length != 1 {
            return Err(Error::DeserializeError(
                "Expected a single key-value pair for enum variant".to_string(),
            ));
        }
        self.de.expect_crlf()?;
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de> serde::de::VariantAccess<'de> for EnumDeserializer<'_, 'de> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Err(Error::DeserializeError(
            "Expected a unit variant, which must be a string".to_string(),
        ))
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::collections::HashMap;

    fn test_u(s: &str, v: i8) {
        assert_eq!(from_str::<u8>(s).unwrap(), v as u8, "u8");
        assert_eq!(from_str::<u16>(s).unwrap(), v as u16, "u16");
        assert_eq!(from_str::<u32>(s).unwrap(), v as u32, "u32");
        assert_eq!(from_str::<u64>(s).unwrap(), v as u64, "u64");
        assert_eq!(from_str::<usize>(s).unwrap(), v as usize, "usize");
        assert!(from_str::<u128>(s).is_err(), "u128");
    }

    fn test_i(s: &str, v: i8) {
        assert_eq!(from_str::<i8>(s).unwrap(), v, "i8");
        assert_eq!(from_str::<i16>(s).unwrap(), v as i16, "i16");
        assert_eq!(from_str::<i32>(s).unwrap(), v as i32, "i32");
        assert_eq!(from_str::<i64>(s).unwrap(), v as i64, "i64");
        assert_eq!(from_str::<isize>(s).unwrap(), v as isize, "isize");
        assert!(from_str::<i128>(s).is_err(), "i128");
    }

    #[test]
    fn test_number() {
        // Test unsigned integers
        test_u(":42\r\n", 42);
        test_u(":+42\r\n", 42);

        // Test signed integers
        test_i(":42\r\n", 42);
        test_i(":+42\r\n", 42);
        test_i(":-42\r\n", -42);
    }

    #[test]
    fn test_big_number() {
        // Test Resp BigNumber -> unsigned integers
        test_u("(42\r\n", 42);
        test_u("(+42\r\n", 42);

        // Test Resp BigNumber -> signed integers
        test_i("(42\r\n", 42);
        test_i("(+42\r\n", 42);
        test_i("(-42\r\n", -42);
    }

    #[test]
    fn test_float() {
        let raw = ",3.1\r\n";
        assert_eq!(from_str::<f32>(raw).unwrap(), 3.1, "f32 unsigned");
        assert_eq!(from_str::<f64>(raw).unwrap(), 3.1, "f64 unsigned");
        let raw = ",+3.1\r\n";
        assert_eq!(from_str::<f32>(raw).unwrap(), 3.1, "f32 plus");
        assert_eq!(from_str::<f64>(raw).unwrap(), 3.1, "f64 plus");
        let raw = ",-3.1\r\n";
        assert_eq!(from_str::<f32>(raw).unwrap(), -3.1, "f32 minus");
        assert_eq!(from_str::<f64>(raw).unwrap(), -3.1, "f64 minus");
        let raw = ",2e20\r\n";
        assert_eq!(from_str::<f32>(raw).unwrap(), 2e20, "f32");
        assert_eq!(from_str::<f64>(raw).unwrap(), 2e20, "f64");
    }

    #[test]
    fn test_string() {
        assert_eq!(
            from_str::<String>("+Hello, World!\r\n").unwrap(),
            "Hello, World!".to_owned(),
            "Simple String"
        );
        assert_eq!(
            from_str::<String>("-Error occurred\r\n").unwrap(),
            "Error occurred".to_owned(),
            "Simple String"
        );
        assert_eq!(
            from_str::<String>("$5\r\nHello\r\n").unwrap(),
            "Hello".to_owned(),
            "Bulk String"
        );
        assert_eq!(
            from_str::<String>("$0\r\n\r\n").unwrap(),
            String::new(),
            "Empty Bulk String"
        );
        assert_eq!(
            from_str::<String>("$-1\r\n").unwrap(),
            String::new(),
            "Null Bulk String"
        );
        assert_eq!(
            from_str::<String>("!5\r\nError\r\n").unwrap(),
            "Error".to_owned(),
            "Bulk Error"
        );
        assert_eq!(
            from_str::<String>("=8\r\nVerbatim\r\n").unwrap(),
            "Verbatim".to_owned(),
            "Verbatim String"
        );

        // Test parsing numbers as strings
        assert_eq!(
            from_str::<String>(":123\r\n").unwrap(),
            "123".to_owned(),
            "Integer as String"
        );
        assert_eq!(
            from_str::<String>("(123\r\n").unwrap(),
            "123".to_owned(),
            "BigNumber as String"
        );
        assert_eq!(
            from_str::<String>(",123\r\n").unwrap(),
            "123".to_owned(),
            "Float as String"
        );
    }

    #[test]
    fn test_array() {
        assert_eq!(
            from_str::<Vec<String>>("*2\r\n$5\r\nHello\r\n$5\r\nWorld\r\n").unwrap(),
            vec!["Hello".to_owned(), "World".to_owned()],
            "Array of Strings"
        );

        assert_eq!(
            from_str::<Vec<i32>>("*2\r\n:-1\r\n:-2\r\n").unwrap(),
            vec![-1, -2],
            "Array of Integers"
        );

        assert_eq!(
            from_str::<Vec<Option<u8>>>("*2\r\n:1\r\n_\r\n").unwrap(),
            vec![Some(1), None],
            "Array of Option<u8>"
        );
    }

    #[test]
    fn test_map() {
        let raw = "%2\r\n+key1\r\n+value1\r\n+key2\r\n+value2\r\n";
        let expected = HashMap::from([
            ("key1".to_owned(), "value1".to_owned()),
            ("key2".to_owned(), "value2".to_owned()),
        ]);
        assert_eq!(from_str::<HashMap<String, String>>(raw).unwrap(), expected);
    }

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<String>,
            opt: Option<f64>,
        }

        let raw = "%3\r\n+int\r\n:1\r\n+seq\r\n*2\r\n+a\r\n+b\r\n+opt\r\n,3.1\r\n";
        let mut expected = Test {
            int: 1,
            seq: vec!["a".to_owned(), "b".to_owned()],
            opt: Some(3.1),
        };
        assert_eq!(expected, from_str(raw).unwrap());
        let raw = "%3\r\n+int\r\n:1\r\n+seq\r\n*2\r\n+a\r\n+b\r\n+opt\r\n_\r\n";
        expected.opt = None;
        assert_eq!(expected, from_str::<Test>(raw).unwrap());
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            AnotherUnit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let raw = "+Unit\r\n";
        let expected = E::Unit;
        assert_eq!(expected, from_str(raw).unwrap());

        let raw = "+AnotherUnit\r\n";
        let expected = E::AnotherUnit;
        assert_eq!(expected, from_str(raw).unwrap());

        let raw = "%1\r\n+Newtype\r\n:1\r\n";
        let expected = E::Newtype(1);
        assert_eq!(expected, from_str(raw).unwrap());

        let raw = "%1\r\n+Tuple\r\n*2\r\n:1\r\n:2\r\n";
        let expected = E::Tuple(1, 2);
        assert_eq!(expected, from_str(raw).unwrap());

        let raw = "%1\r\n+Struct\r\n%1\r\n+a\r\n:1\r\n";
        let expected = E::Struct { a: 1 };
        assert_eq!(expected, from_str(raw).unwrap());
    }
}
