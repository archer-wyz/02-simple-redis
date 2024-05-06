use super::*;
use anyhow::Result;
use bytes::{Buf, BytesMut};
use std::ops::{Deref, DerefMut};
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct RespArray(Vec<RespFrame>);

impl RespEncode for RespArray {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(DEFAULT_CAPACITY);
        buf.extend_from_slice(format!("*{}\r\n", self.len()).as_bytes());
        for frame in self.iter() {
            buf.extend_from_slice(frame.encode().as_slice());
        }
        buf
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (len, pos) = parse_length(data)?;
        data.advance(pos);
        let mut ra = RespArray::new();
        for _ in 0..len {
            let frame = RespFrame::decode(data).map_err(|e| RespError::RespWrappedError {
                typ: "array".to_string(),
                err: Box::new(e),
            })?;
            ra.push(frame);
        }
        Ok(ra.into())
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RespArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RespArray {
    pub fn new() -> Self {
        RespArray(Vec::new())
    }

    pub fn with_vec(v: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(v.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::BulkString;
    #[test]
    fn test_resp_array_encode() {
        let mut ra = RespArray::new();
        ra.push(Double(1.0));
        ra.push(BulkString::new("hello world").into());
        let res = ra.encode();
        assert_eq!(res, b"*2\r\n,1.0\r\n$11\r\nhello world\r\n");
    }

    #[test]
    fn test_resp_array_decode() {
        let mut data = BytesMut::from("*2\r\n+hello\r\n+world\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b"*2\r\n+hello\r\n+world\r\n");

        let mut data = BytesMut::from("*3\r\n+hello\r\n+world\r\n$11\r\nhello world\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(
            res.encode(),
            b"*3\r\n+hello\r\n+world\r\n$11\r\nhello world\r\n"
        );

        let mut data = BytesMut::from("*3\r\n+hello\r\n$11\r\nhello world\r\n#t\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(
            res.encode(),
            b"*3\r\n+hello\r\n$11\r\nhello world\r\n#t\r\n"
        );
    }

    #[test]
    fn test_camp_array_decode() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::with_vec([b"set".into(), b"hello".into()]).into()
        );

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf.clone());
        assert_eq!(
            ret.unwrap_err(),
            RespError::RespWrappedError {
                typ: "array".to_string(),
                err: Box::new(RespError::RespIsEmpty),
            }
        );

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(
            frame,
            RespArray::with_vec([b"set".into(), b"hello".into()]).into()
        );

        Ok(())
    }
}
