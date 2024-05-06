use super::*;
use bytes::Buf;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespNullArray();

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespNull();

impl RespEncode for RespNull {
    fn encode(&self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespEncode for RespNullArray {
    fn encode(&self) -> Vec<u8> {
        b"-1\r\n".to_vec()
    }
}

impl RespDecode for RespNull {
    const PREFIX: &'static str = "_";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (s, pos) = split_cr_lf(data)?;
        if s.len() != 0 {
            return Err(RespError::RespNotComplete("Invalid null".to_string()));
        }
        Ok(RespNull())
    }
}

impl RespDecode for RespNullArray {
    const PREFIX: &'static str = "*";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (s, pos) = split_cr_lf(data)?;
        if s.len() != 2 || s[0] != b'-' || s[1] != b'1' {
            return Err(RespError::RespNotComplete("Invalid null array".to_string()));
        }
        data.advance(pos);
        Ok(RespNullArray())
    }
}
