use super::*;
use bytes::{Buf, BytesMut};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct SimpleString(pub(crate) String);

impl RespEncode for SimpleString {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(DEFAULT_CAPACITY);
        buf.extend_from_slice(b"+");
        buf.extend_from_slice(self.as_bytes());
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

// simple string: "+OK\r\n"
//
impl RespDecode for SimpleString {
    const PREFIX: &'static str = "+";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (s, len) = split_cr_lf(data)?;
        let res = SimpleString::new(String::from_utf8_lossy(s).to_string());
        data.advance(len);
        Ok(res)
    }
}

impl Deref for SimpleString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SimpleString {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleString(s.into())
    }
}

impl<T: Into<String>> From<T> for SimpleString {
    fn from(s: T) -> Self {
        SimpleString::new(s.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_simple_string_encode() {
        let ss = SimpleString::new("hello world");
        let res = ss.encode();
        assert_eq!(res, b"+hello world\r\n");
    }

    #[test]
    fn test_simple_string_decode() {
        let mut data = BytesMut::from("+hello world\r\n");
        let res = SimpleString::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b"+hello world\r\n");
    }
}
