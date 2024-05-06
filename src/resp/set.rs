use super::*;
use anyhow::Result;
use bytes::Buf;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespSet(pub(crate) Vec<RespFrame>);

impl RespEncode for RespSet {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(DEFAULT_CAPACITY);
        buf.extend_from_slice(b"~");
        buf.extend_from_slice(self.len().to_string().as_bytes());
        buf.extend_from_slice(b"\r\n");
        for frame in self.iter() {
            buf.extend_from_slice(frame.encode().as_slice());
        }
        buf
    }
}

impl RespSet {
    pub fn new() -> Self {
        RespSet(Vec::new())
    }
    pub fn with_vec(v: impl Into<Vec<RespFrame>>) -> Self {
        RespSet(v.into())
    }
}

impl Default for RespSet {
    fn default() -> Self {
        RespSet::new()
    }
}

impl RespDecode for RespSet {
    const PREFIX: &'static str = "~";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (len, pos) = parse_length(data)?;
        data.advance(pos);
        let mut rs = RespSet::new();
        for _ in 0..len {
            let frame = RespFrame::decode(data)?;
            rs.push(frame)
        }
        Ok(rs)
    }
}

impl Deref for RespSet {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::BulkString;
    #[test]
    fn test_resp_set_encode() {
        let mut rs = RespSet::new();
        rs.push(1.0f64.into());
        rs.push(BulkString::new(b"hello world".to_vec()).into());
        let res = rs.encode();
        assert_eq!(res, b"~2\r\n,1.0\r\n$11\r\nhello world\r\n");
    }

    #[test]
    fn test_resp_set_decode() {
        let mut data = BytesMut::from("~2\r\n,1.0\r\n$11\r\nhello world\r\n");
        let res = RespSet::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b"~2\r\n,1.0\r\n$11\r\nhello world\r\n");
    }

    #[test]
    fn test_set_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"~2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespSet::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespSet::with_vec(vec![
                BulkString::new(b"set".to_vec()).into(),
                BulkString::new(b"hello".to_vec()).into()
            ])
        );

        Ok(())
    }
}
