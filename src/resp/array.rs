use super::*;
use anyhow::Result;
use bytes::{Buf, BytesMut};
use macro_definitions::AutoDeref;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd, AutoDeref)]
#[deref(mutable)]
pub struct RespArray(pub(crate) Option<Vec<RespFrame>>);

impl RespEncode for RespArray {
    fn encode(&self) -> Vec<u8> {
        match self.0.as_ref() {
            None => b"*-1\r\n".to_vec(),
            Some(v) => {
                let mut buf = Vec::with_capacity(DEFAULT_CAPACITY);
                buf.extend_from_slice(b"*");
                buf.extend_from_slice(v.len().to_string().as_bytes());
                buf.extend_from_slice(b"\r\n");
                for frame in v.iter() {
                    buf.extend_from_slice(frame.encode().as_slice());
                }
                buf
            }
        }
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (len, pos) = parse_length(data)?;
        if len == -1 {
            data.advance(pos);
            return Ok(RespArray::new_null());
        }

        data.advance(pos);
        let mut ra = Vec::with_capacity(len as usize);
        for _ in 0..len {
            let frame = RespFrame::decode(data).map_err(|e| e.map_not_complete())?;
            ra.push(frame);
        }
        Ok(RespArray::with_vec(ra))
    }
}

impl Display for RespArray {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0.as_ref() {
            None => write!(f, "null"),
            Some(v) => {
                write!(f, "[")?;
                for (i, frame) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", frame)?;
                }
                write!(f, "]")
            }
        }
    }
}

// impl Deref for RespArray {
//     type Target = Option<Vec<RespFrame>>;
//
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
//
// impl DerefMut for RespArray {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

impl RespArray {
    pub fn new_null() -> Self {
        RespArray(None)
    }
    pub fn new() -> Self {
        RespArray(Some(Vec::new()))
    }

    pub fn with_vec(v: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(Some(v.into()))
    }

    pub fn try_push(&mut self, frame: impl Into<RespFrame>) -> Result<()> {
        match self.0.as_mut() {
            Some(v) => {
                v.push(frame.into());
                Ok(())
            }
            None => Err(anyhow::anyhow!("Can't push to null array")),
        }
    }
}

impl Default for RespArray {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::BulkString;

    #[test]
    fn test_resp_array_encode() -> Result<()> {
        let mut ra = RespArray::new();
        ra.try_push(1.0f64)?;
        ra.try_push(BulkString::new("hello world"))?;
        let res = ra.encode();
        println!("{:?}", String::from_utf8_lossy(res.as_slice()).to_string());
        assert_eq!(res, b"*2\r\n,1.0\r\n$11\r\nhello world\r\n");
        Ok(())
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
        assert_eq!(frame, RespArray::with_vec([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf.clone());
        assert_eq!(ret.unwrap_err(), RespError::RespNotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::with_vec([b"set".into(), b"hello".into()]));
        Ok(())
    }

    #[test]
    fn test_resp_array_encode_null() {
        let ra = RespArray::new_null();
        let res = ra.encode();
        assert_eq!(res, b"*-1\r\n");
    }

    #[test]
    fn test_resp_array_decode_null() {
        let mut data = BytesMut::from("*-1\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_resp_array_not_complete() {
        let mut data = BytesMut::from("*2\r\n+hello\r\n");
        let res = RespArray::decode(&mut data);
        assert_eq!(res.unwrap_err(), RespError::RespNotComplete);

        let mut data = BytesMut::from("*2\r\n+hello\r");
        let res = RespArray::decode(&mut data);
        assert_eq!(res.unwrap_err(), RespError::RespNotComplete);

        let mut data = BytesMut::from("*2\r\n+hello\r\n+abc");
        let res = RespArray::decode(&mut data);
        assert_eq!(res.unwrap_err(), RespError::RespNotComplete);
    }
}
