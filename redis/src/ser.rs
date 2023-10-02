use std::fmt;
use std::io;

use crate::error::{Error, Result};
use crate::types::RedisType;

use serde::{ser, Serialize};

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut buf = Vec::new();
    value.serialize(&mut Serializer::new(&mut buf))?;
    let s = String::from_utf8(buf.to_vec()).unwrap();
    Ok(s)
}

#[cfg(test)]
mod test_derive {
    use super::*;
    use serde::Serialize;

    #[test]
    fn test_derive() {
        #[derive(Serialize)]
        struct Point {
            x: i32,
            y: i32,
        }

        let point = Point { x: 1, y: 2 };
        let serialized = to_string(&point).unwrap();
        println!("serialized = {:?}", serialized);
    }
}

#[cfg(test)]
mod to_string_test {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn boolean_to_string() {
        let value = true;
        assert_eq!("#t\r\n", to_string(&value).unwrap());
    }

    #[test]
    fn integer_to_string() {
        let value = 10;
        assert_eq!(":10\r\n", to_string(&value).unwrap());
    }

    #[test]
    fn big_number_to_string() {
        // TODO:
    }

    #[test]
    fn double_to_string() {
        let value = 101.0f64;
        assert_eq!(",101.0\r\n", to_string(&value).unwrap());
    }

    #[test]
    fn simple_string_to_string() {
        let value = '+';
        assert_eq!("++\r\n", to_string(&value).unwrap());
    }

    #[test]
    fn bulk_string_to_string() {
        let value = "123456789";
        assert_eq!(format!("$9\r\n{value}\r\n"), to_string(&value).unwrap());
    }

    #[test]
    fn verbatim_string_to_string() {
        // TODO:
    }

    #[test]
    fn simple_error_to_string() {
        // TODO:
    }

    #[test]
    fn bulk_error_to_string() {
        // TODO:
    }

    #[test]
    fn null_to_string() {
        assert_eq!("_\r\n", to_string(&None::<String>).unwrap());
    }

    #[test]
    fn array_to_string() {
        let value = vec!["a", "b"];
        assert_eq!("*2\r\n$1\r\na\r\n$1\r\nb\r\n", to_string(&value).unwrap());
    }

    #[test]
    fn map_to_string() {
        let mut value = HashMap::new();
        value.insert("k1", "v1");
        value.insert("k2", "v2");

        let result1 =
            "%2\r\n$2\r\nk1\r\n$2\r\nv1\r\n$2\r\nk2\r\n$2\r\nv2\r\n".to_string();
        let result2 =
            "%2\r\n$2\r\nk2\r\n$2\r\nv2\r\n$2\r\nk1\r\n$2\r\nv1\r\n".to_string();
        let result = to_string(&value).unwrap();
        let matched = if result == result1 {
            true
        } else if result == result2 {
            true
        } else {
            false
        };
        assert!(matched);
    }

    #[test]
    fn set_to_string() {
        // TODO:
    }

    #[test]
    fn push_to_string() {
        // TODO:
    }
}

/// 用于根据 [RESP](https://redis.io/docs/reference/protocol-spec/) 协议执行进行序列化
pub struct Serializer<'a, W>
where
    W: ?Sized + io::Write,
{
    w: &'a mut W,
}

impl<'a, W: io::Write> Serializer<'a, W> {
    pub fn new(w: &'a mut W) -> Self {
        Self { w }
    }

    fn write_integer(&mut self, value: i64) -> Result<()> {
        write!(self.w, "{}{}{}", RedisType::INTEGER, value, RedisType::CRLF)?;
        Ok(())
    }

    fn write_big_number(&mut self, value: impl fmt::Display) -> Result<()> {
        write!(
            self.w,
            "{}{}{}",
            RedisType::BIG_NUMBER,
            value,
            RedisType::CRLF
        )?;
        Ok(())
    }

    fn write_double(&mut self, value: f64) -> Result<()> {
        write!(
            self.w,
            "{}{:?}{}",
            RedisType::DOUBLE,
            value,
            RedisType::CRLF
        )?;
        Ok(())
    }

    fn write_boolean(&mut self, value: bool) -> Result<()> {
        write!(
            self.w,
            "{}{}{}",
            RedisType::BOOLEAN,
            if value { 't' } else { 'f' },
            RedisType::CRLF
        )?;
        Ok(())
    }

    fn write_simple_string(&mut self, value: impl fmt::Display) -> Result<()> {
        write!(
            self.w,
            "{}{}{}",
            RedisType::SIMPLE_STRING,
            value,
            RedisType::CRLF
        )?;
        Ok(())
    }

    fn write_bulk_string(&mut self, value: impl fmt::Display) -> Result<()> {
        let value = value.to_string();
        write!(
            self.w,
            "{}{}{}",
            RedisType::BULK_STRING,
            value.len(),
            RedisType::CRLF
        )?;
        write!(self.w, "{}{}", value, RedisType::CRLF)?;
        Ok(())
    }

    fn write_null(&mut self) -> Result<()> {
        write!(self.w, "{}{}", RedisType::NULL, RedisType::CRLF)?;
        Ok(())
    }

    fn write_array_header(&mut self, len: Option<usize>) -> Result<&mut Self> {
        let len = len.unwrap_or(0);
        write!(self.w, "{}{}{}", RedisType::ARRAY, len, RedisType::CRLF)?;
        Ok(self)
    }

    fn write_map_header(&mut self, len: Option<usize>) -> Result<&mut Self> {
        let len = len.unwrap_or(0);
        write!(self.w, "{}{}{}", RedisType::MAP, len, RedisType::CRLF)?;
        Ok(self)
    }
}

impl<'a, W: io::Write> ser::SerializeSeq for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeTuple for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeTupleStruct for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeTupleVariant for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeMap for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(&mut **self)
    }

    fn serialize_entry<K: ?Sized, V: ?Sized>(
        &mut self,
        key: &K,
        value: &V,
    ) -> Result<Self::Ok>
    where
        K: Serialize,
        V: Serialize,
    {
        self.serialize_key(key)?;
        self.serialize_value(value)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeStruct for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeStructVariant for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        self.write_simple_string(key)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(())
    }
}

impl<'a, W: io::Write> ser::Serializer for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.write_boolean(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.write_integer(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.write_integer(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.write_integer(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.write_integer(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.write_integer(v as i64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.write_integer(v as i64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.write_big_number(v as i64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.write_big_number(v.to_string())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.write_double(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.write_double(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        match v {
            '\r' | '\n' => self.write_bulk_string(v),
            _ => self.write_simple_string(v),
        }
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        // TODO: 长度, 包含的字符
        // TODO: 对于 big number 要做区分
        self.write_bulk_string(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        let v = std::str::from_utf8(v).unwrap();
        self.write_bulk_string(v)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.write_null()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: Serialize + ?Sized,
    {
        // TODO: 检查这里是否正确
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.write_array_header(None).map(|_| ())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.write_map_header(Some(0)).map(|_| ())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.write_bulk_string(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.write_array_header(len)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.write_array_header(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.write_array_header(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.write_array_header(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.write_map_header(len)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.write_map_header(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.write_map_header(Some(len))
    }
}
