use super::*;
use bytes::Buf;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespNull();

impl RespEncode for RespNull {
    fn encode(&self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (s, pos) = split_cr_lf(data)?;
        if s.is_empty() {
            return Err(RespError::RespInvalid("Invalid null".to_string()));
        }
        data.advance(pos);
        Ok(RespNull())
    }
}

impl Display for RespNull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}
