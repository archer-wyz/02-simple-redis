use super::*;
use bytes::Buf;
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BulkString(pub(crate) Vec<u8>);

// $<length>\r\n<data>\r\n
impl RespEncode for BulkString {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(DEFAULT_CAPACITY);
        buf.extend_from_slice(format!("${}\r\n", self.len()).as_bytes());
        buf.extend_from_slice(self.as_slice());
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespDecode for BulkString {
    const PREFIX: &'static str = "$";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (len, pos) = parse_length(data)?;
        data.advance(pos);
        let (s, pos) = split_cr_lf(data)?;
        if s.len() != len {
            return Err(RespError::RespNotEqualLength {
                expected: len,
                decoded: s.len(),
            });
        }
        let res = BulkString::new(s).into();
        data.advance(pos);
        Ok(res)
    }
}

impl Deref for BulkString {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for BulkString {
    fn from(s: &str) -> Self {
        BulkString(s.as_bytes().to_vec())
    }
}

impl BulkString {
    pub fn new(s: impl Into<Vec<u8>>) -> Self {
        BulkString(s.into())
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
}
