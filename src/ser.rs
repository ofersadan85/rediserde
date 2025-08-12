#![allow(clippy::missing_errors_doc)]

use crate::{CRLF, Error, Result, resp::RespDataKind};

#[derive(Debug, Default)]
pub struct Serializer {
    output: Vec<u8>,
}

impl Serializer {
    #[must_use]
    pub const fn new() -> Self {
        Self { output: Vec::new() }
    }

    /// Inspect the current output for debugging purposes.
    #[allow(dead_code)]
    fn inspect(&self) {
        let input_lossy = String::from_utf8_lossy(&self.output);
        dbg!(input_lossy);
    }
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>>
where
    T: serde::Serialize,
{
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: serde::Serialize,
{
    let mut serializer = Serializer::new();
    value.serialize(&mut serializer)?;
    Ok(String::from_utf8(serializer.output)?)
}

impl serde::Serializer for &mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    /// #<t|f>\r\n
    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.output.push(RespDataKind::Boolean.to_prefix_bytes());
        self.output.push(if v { b't' } else { b'f' });
        self.output.extend_from_slice(CRLF);
        Ok(())
    }

    /// Uses `self.serialize_i64` internally.
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    /// Uses `self.serialize_i64` internally.
    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    /// Uses `self.serialize_i64` internally.
    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.output.push(RespDataKind::Integer.to_prefix_bytes());
        self.output.extend_from_slice(v.to_string().as_bytes());
        self.output.extend_from_slice(CRLF);
        Ok(())
    }

    /// Uses `self.serialize_i64` internally.
    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    /// Uses `self.serialize_i64` internally.
    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    /// Uses `self.serialize_i64` internally.
    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    /// RESP Integer is at most i64, so a u64 will be serialized as a `BigNumber`.
    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.output.push(RespDataKind::BigNumber.to_prefix_bytes());
        self.output.extend_from_slice(v.to_string().as_bytes());
        self.output.extend_from_slice(CRLF);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        // Does *not* use `self.serialize_f64` internally to avoid precision loss.
        self.output.push(RespDataKind::Float.to_prefix_bytes());
        self.output.extend_from_slice(v.to_string().as_bytes());
        self.output.extend_from_slice(CRLF);
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.output.push(RespDataKind::Float.to_prefix_bytes());
        self.output.extend_from_slice(v.to_string().as_bytes());
        self.output.extend_from_slice(CRLF);
        Ok(())
    }

    /// Uses `self.serialize_bytes` internally.
    /// Always serializes as a bulk string and not a simple string.
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_str(v.to_string().as_str())
    }

    /// Uses `self.serialize_bytes` internally.
    /// Always serializes as a bulk string and not a simple string.
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.serialize_bytes(v.as_bytes())
    }

    /// Always serializes as a bulk string and not a simple string.
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        // $<length>\r\n<data>\r\n
        self.output.push(RespDataKind::BulkString.to_prefix_bytes());
        self.output
            .extend_from_slice(v.len().to_string().as_bytes());
        self.output.extend_from_slice(CRLF);
        self.output.extend_from_slice(v);
        self.output.extend_from_slice(CRLF);
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        // _\r\n
        // As this is known to be a constant, we avoid multiple push/extend calls.
        self.output.extend_from_slice(b"_\r\n");
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    /// Uses `self.serialize_none` internally.
    fn serialize_unit(self) -> Result<Self::Ok> {
        self.serialize_none()
    }

    /// Uses `self.serialize_none` internally.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_none()
    }

    /// Uses `self.serialize_str` internally.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    /// Ignores the newtype wrapper, serializes the data directly
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(self)
    }

    /// Serializes a newtype struct as a map with a single key-value pair.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + serde::Serialize,
    {
        self.output.push(RespDataKind::Map.to_prefix_bytes());
        self.output.push(b'1'); // Single key-value pair
        self.output.extend_from_slice(CRLF);
        self.serialize_str(variant)?;
        value.serialize(self)
    }

    /// Serializes a sequence as an array.
    /// An empty sequence is serialized as *0\r\n
    /// A null sequence is serialized as *-1\r\n, and will be output for a sequence of unknown length.
    /// A non-empty sequence is serialized as `*<length>\r\n<data>`
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output.push(RespDataKind::Array.to_prefix_bytes());
        match len {
            Some(l) => {
                self.output.extend_from_slice(l.to_string().as_bytes());
            }
            None => {
                self.output.extend_from_slice(b"-1");
            }
        }
        self.output.extend_from_slice(CRLF);
        Ok(self)
    }

    /// Serializes a tuple as a sequence.
    /// Uses `self.serialize_seq` internally.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    /// Serializes a tuple struct as a sequence.
    /// Uses `self.serialize_seq` internally.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    /// Serializes a tuple variant as map from variant to a sequence.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.output.push(RespDataKind::Map.to_prefix_bytes());
        self.output.extend_from_slice(b"1"); // Single key-value pair
        self.output.extend_from_slice(CRLF);
        self.serialize_str(variant)?;
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        // %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
        let len = len.ok_or_else(|| {
            Error::SerializeError("Cannot serialize a map with unknown length".to_string())
        })?;
        self.output.push(RespDataKind::Map.to_prefix_bytes());
        self.output.extend_from_slice(len.to_string().as_bytes());
        self.output.extend_from_slice(CRLF);
        Ok(self)
    }

    /// A struct is serialized exactly like a map
    /// Uses `self.serialize_map` internally
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    /// Serializes as a map from the variant to a struct
    /// Uses `self.serialize_struct` internally
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output.push(RespDataKind::Map.to_prefix_bytes());
        self.output.extend_from_slice(b"1"); // Single key-value pair
        self.output.extend_from_slice(CRLF);
        self.serialize_str(variant)?;
        self.serialize_struct(name, len)
    }
}

impl serde::ser::SerializeSeq for &mut Serializer {
    type Ok = ();
    type Error = Error;

    /// There's no separation between values in a RESP array
    /// So each value can serialize itself
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    /// There is no ending output to a RESP array, adds nothing
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl serde::ser::SerializeTuple for &mut Serializer {
    type Ok = ();
    type Error = Error;

    /// Identical to `SerializeSeq` implementation
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    /// Identical to `SerializeSeq` implementation
    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    /// Identical to `SerializeSeq` implementation
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    /// Identical to `SerializeSeq` implementation
    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    /// Identical to `SerializeSeq` implementation
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    /// Identical to `SerializeSeq` implementation
    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeMap for &mut Serializer {
    type Ok = ();
    type Error = Error;

    /// Keys and values serialize themselves
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        key.serialize(&mut **self)
    }

    /// Keys and values serialize themselves
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    /// There is no ending output to a RESP map, adds nothing
    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl serde::ser::SerializeStruct for &mut Serializer {
    type Ok = ();
    type Error = Error;

    /// Uses `SerializeMap` implementation internally
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)?;
        Ok(())
    }

    /// Uses `SerializeMap` implementation internally
    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeMap::end(self)
    }
}

impl serde::ser::SerializeStructVariant for &mut Serializer {
    type Ok = ();
    type Error = Error;

    // Uses `SerializeMap` implementation internally
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + serde::Serialize,
    {
        serde::ser::SerializeMap::serialize_key(self, key)?;
        serde::ser::SerializeMap::serialize_value(self, value)?;
        Ok(())
    }

    /// Uses `SerializeMap` implementation internally
    fn end(self) -> Result<Self::Ok> {
        serde::ser::SerializeMap::end(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use std::collections::HashMap;

    fn test_u(val: u8, expected: &str) {
        assert_eq!(to_string(&val).unwrap(), expected, "u8");
        assert_eq!(to_string(&(val as u16)).unwrap(), expected, "u16");
        assert_eq!(to_string(&(val as u32)).unwrap(), expected, "u32");
        assert_eq!(
            to_string(&(val as u64)).unwrap(),
            format!("({}\r\n", val),
            "u64"
        );
    }

    fn test_i(val: i8, expected: &str) {
        assert_eq!(to_string(&val).unwrap(), expected, "i8");
        assert_eq!(to_string(&(val as i16)).unwrap(), expected, "i16");
        assert_eq!(to_string(&(val as i32)).unwrap(), expected, "i32");
        assert_eq!(to_string(&(val as i64)).unwrap(), expected, "i64");
        assert_eq!(to_string(&(val as isize)).unwrap(), expected, "isize");
    }

    #[test]
    fn test_number() {
        test_u(42, ":42\r\n");
        test_i(42, ":42\r\n");
        test_i(-42, ":-42\r\n");
    }

    #[test]
    fn test_big_number() {
        let expected = "(12345678901234567890\r\n";
        let val = 12345678901234567890_u64;
        assert_eq!(to_string(&val).unwrap(), expected);
        assert_eq!(to_string(&(val as usize)).unwrap(), expected, "usize");
    }

    #[test]
    fn test_float() {
        assert_eq!(to_string(&3.1_f32).unwrap(), ",3.1\r\n", "plus f32");
        assert_eq!(to_string(&3.1_f64).unwrap(), ",3.1\r\n", "plus f64");
        assert_eq!(to_string(&-3.1_f32).unwrap(), ",-3.1\r\n", "plus f32");
        assert_eq!(to_string(&-3.1_f64).unwrap(), ",-3.1\r\n", "plus f64");
        assert_eq!(
            to_string(&2e20_f64).unwrap(),
            ",200000000000000000000\r\n",
            "exp f64"
        );
        assert_eq!(
            to_string(&2e-20_f64).unwrap(),
            ",0.00000000000000000002\r\n",
            "neg exp f64"
        );
    }

    #[test]
    fn test_string() {
        assert_eq!(
            to_string(&"Hello, World!").unwrap(),
            "$13\r\nHello, World!\r\n"
        );
        assert_eq!(
            to_string(&String::from("Hello")).unwrap(),
            "$5\r\nHello\r\n"
        );
        assert_eq!(to_string(&String::new()).unwrap(), "$0\r\n\r\n");
    }

    #[test]
    fn test_array() {
        let arr = vec!["Hello".to_owned(), "World".to_owned()];
        assert_eq!(
            to_string(&arr).unwrap(),
            "*2\r\n$5\r\nHello\r\n$5\r\nWorld\r\n"
        );

        let arr = vec![-1i32, -2i32];
        assert_eq!(to_string(&arr).unwrap(), "*2\r\n:-1\r\n:-2\r\n");

        let arr = vec![Some(1u8), None];
        assert_eq!(to_string(&arr).unwrap(), "*2\r\n:1\r\n_\r\n");
    }

    #[test]
    fn test_map() {
        let mut map = HashMap::new();
        map.insert("key1".to_owned(), "value1".to_owned());
        map.insert("key2".to_owned(), "value2".to_owned());
        let out = to_string(&map).unwrap();
        // Order is not guaranteed, so check both possibilities
        let expected1 = "%2\r\n$4\r\nkey1\r\n$6\r\nvalue1\r\n$4\r\nkey2\r\n$6\r\nvalue2\r\n";
        let expected2 = "%2\r\n$4\r\nkey2\r\n$6\r\nvalue2\r\n$4\r\nkey1\r\n$6\r\nvalue1\r\n";
        assert!(out == expected1 || out == expected2);
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize, PartialEq, Debug)]
        struct Test {
            int: u32,
            seq: Vec<String>,
            opt: Option<f64>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a".to_owned(), "b".to_owned()],
            opt: Some(3.1),
        };
        let out = to_string(&test).unwrap();
        // The order of fields is not guaranteed, so check for all possibilities
        let expected1 = "%3\r\n$3\r\nint\r\n:1\r\n$3\r\nseq\r\n*2\r\n$1\r\na\r\n$1\r\nb\r\n$3\r\nopt\r\n,3.1\r\n";
        let expected2 = "%3\r\n$3\r\nseq\r\n*2\r\n$1\r\na\r\n$1\r\nb\r\n$3\r\nint\r\n:1\r\n$3\r\nopt\r\n,3.1\r\n";
        let expected3 = "%3\r\n$3\r\nopt\r\n,3.1\r\n$3\r\nint\r\n:1\r\n$3\r\nseq\r\n*2\r\n$1\r\na\r\n$1\r\nb\r\n";
        assert!(out == expected1 || out == expected2 || out == expected3);

        let test = Test {
            int: 1,
            seq: vec!["a".to_owned(), "b".to_owned()],
            opt: None,
        };
        let out = to_string(&test).unwrap();
        let expected1 =
            "%3\r\n$3\r\nint\r\n:1\r\n$3\r\nseq\r\n*2\r\n$1\r\na\r\n$1\r\nb\r\n$3\r\nopt\r\n_\r\n";
        let expected2 =
            "%3\r\n$3\r\nseq\r\n*2\r\n$1\r\na\r\n$1\r\nb\r\n$3\r\nint\r\n:1\r\n$3\r\nopt\r\n_\r\n";
        let expected3 =
            "%3\r\n$3\r\nopt\r\n_\r\n$3\r\nint\r\n:1\r\n$3\r\nseq\r\n*2\r\n$1\r\na\r\n$1\r\nb\r\n";
        assert!(out == expected1 || out == expected2 || out == expected3);
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize, PartialEq, Debug)]
        enum E {
            Unit,
            AnotherUnit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let e = E::Unit;
        let out = to_string(&e).unwrap();
        assert_eq!(out, "$4\r\nUnit\r\n");

        let e = E::AnotherUnit;
        let out = to_string(&e).unwrap();
        assert_eq!(out, "$11\r\nAnotherUnit\r\n");

        let e = E::Newtype(1);
        let out = to_string(&e).unwrap();
        assert_eq!(out, "%1\r\n$7\r\nNewtype\r\n:1\r\n");

        let e = E::Tuple(1, 2);
        let out = to_string(&e).unwrap();
        assert_eq!(out, "%1\r\n$5\r\nTuple\r\n*2\r\n:1\r\n:2\r\n");

        let e = E::Struct { a: 1 };
        let out = to_string(&e).unwrap();
        assert_eq!(out, "%1\r\n$6\r\nStruct\r\n%1\r\n$1\r\na\r\n:1\r\n")
    }
}
