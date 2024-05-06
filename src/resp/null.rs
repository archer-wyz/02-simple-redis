use super::*;
use bytes::Buf;

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
            return Err(RespError::RespNotComplete("Invalid null".to_string()));
        }
        data.advance(pos);
        Ok(RespNull())
    }
}
