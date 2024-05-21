use super::*;
use bytes::Buf;
use macro_definitions::AutoDeref;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd, AutoDeref)]
pub struct SimpleError(String);

impl RespEncode for SimpleError {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(DEFAULT_CAPACITY);
        buf.extend_from_slice(b"-");
        buf.extend_from_slice(self.as_bytes());
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespDecode for SimpleError {
    const PREFIX: &'static str = "-";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (s, len) = split_cr_lf(data)?;
        let res = SimpleError::new(String::from_utf8_lossy(s).to_string());
        data.advance(len);
        Ok(res)
    }
}

impl SimpleError {
    pub fn new(s: impl Into<String>) -> Self {
        SimpleError(s.into())
    }
}
impl Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
