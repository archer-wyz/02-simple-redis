use super::*;
use bytes::Buf;

impl RespEncode for bool {
    fn encode(&self) -> Vec<u8> {
        format!("#{}\r\n", if *self { "t" } else { "f" }).into_bytes()
    }
}

impl RespDecode for bool {
    const PREFIX: &'static str = "#";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (s, pos) = split_cr_lf(data)?;
        let res = match s {
            b"t" => true,
            b"f" => false,
            _ => return Err(RespError::RespInvalid("Invalid bool".to_string())),
        };
        data.advance(pos);
        Ok(res)
    }
}
