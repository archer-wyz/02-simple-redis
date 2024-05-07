use super::*;
use anyhow::Result;
use bytes::Buf;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum BulkString {
    Vec(Vec<u8>),
    Null,
}

// $<length>\r\n<data>\r\n
impl RespEncode for BulkString {
    fn encode(&self) -> Vec<u8> {
        match self {
            BulkString::Vec(v) => {
                let mut buf = Vec::with_capacity(DEFAULT_CAPACITY);
                buf.extend_from_slice(format!("${}\r\n", v.len()).as_bytes());
                buf.extend_from_slice(v.as_slice());
                buf.extend_from_slice(b"\r\n");
                buf
            }
            BulkString::Null => {
                let mut buf = Vec::with_capacity(DEFAULT_CAPACITY);
                buf.extend_from_slice(b"$-1\r\n");
                buf
            }
        }
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());

        // get length and move cursor to the end of \r\n
        let (len, pos) = parse_length(data)?;
        data.advance(pos);
        if len == -1 {
            return Ok(BulkString::new_null());
        }

        let (s, pos) = split_cr_lf(data)?;
        if s.len() != len as usize {
            return Err(RespError::RespNotEqualLength {
                expected: len as usize,
                decoded: s.len(),
            });
        }

        let res = BulkString::new(s);
        data.advance(pos);
        Ok(res)
    }
}

// 这样写会有歧义，直接不用
// impl Deref for BulkString {
//     type Target = Vec<u8>;
//
//     fn deref(&self) -> &Self::Target {
//         match self {
//             BulkString::Vec(v) => v,
//             BulkString::Null => &Vec::new()
//         }
//     }
// }

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString::new(s.as_bytes())
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString::Vec(s.into())
    }

    pub fn new_null() -> Self {
        BulkString::Null
    }
}

impl Display for BulkString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BulkString::Vec(v) => write!(f, "{}", String::from_utf8_lossy(v)),
            BulkString::Null => Err(std::fmt::Error),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bulk_string_encode() {
        let bs = BulkString::from("hello world");
        let res = bs.encode();
        assert_eq!(res, b"$11\r\nhello world\r\n");
        let bs2 = BulkString::new("hello world".as_bytes());
        let res2 = bs2.encode();
        assert_eq!(res2, b"$11\r\nhello world\r\n");
    }

    #[test]
    fn test_bulk_string_decode() {
        let mut data = BytesMut::from("$11\r\nhello world\r\n");
        let res = BulkString::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b"$11\r\nhello world\r\n");

        let mut data = BytesMut::from("$10\r\nhello world\r\n");
        let res = BulkString::decode(&mut data).unwrap_err();
        assert_eq!(
            res,
            RespError::RespNotEqualLength {
                expected: 10,
                decoded: 11,
            }
        );
    }

    #[test]
    fn test_bulk_string_decode_null() {
        let mut data = BytesMut::from("$-1\r\n");
        let res = BulkString::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b"$-1\r\n");
    }
}
