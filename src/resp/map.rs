use super::*;
use bytes::Buf;
use macro_definitions::AutoDeref;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, PartialOrd, AutoDeref)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);

// %<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>
impl RespEncode for RespMap {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(format!("%{}\r\n", self.len()).as_bytes());
        for (k, v) in self.iter() {
            buf.extend_from_slice(SimpleString::new(k).encode().as_slice());
            buf.extend_from_slice(v.encode().as_slice());
        }
        buf
    }
}

impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (len, pos) = parse_length(data)?;
        data.advance(pos);
        let mut rm = RespMap::new();
        for _ in 0..len {
            let key = RespFrame::decode(data).map_err(|e| e.map_not_complete())?;
            let value = RespFrame::decode(data).map_err(|e| e.map_not_complete())?;
            match key {
                RespFrame::SimpleString(k) => {
                    rm.insert(k.0, value);
                }
                _ => {
                    return Err(RespError::RespInvalid(
                        "map key must be simple string".to_string(),
                    ))
                }
            }
        }
        Ok(rm)
    }
}

impl Display for RespMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for (i, (k, v)) in self.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", k, v)?;
        }
        write!(f, "}}")
    }
}

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<RespFrame>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn with_map(m: impl Into<BTreeMap<String, RespFrame>>) -> Self {
        RespMap(m.into())
    }

    pub fn with_vec(v: impl Into<Vec<(String, RespFrame)>>) -> Self {
        RespMap(v.into().into_iter().collect())
    }
}

impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_resp_map_encode() {
        let mut rm = RespMap::new();
        rm.insert("hello", BulkString::new("world"));
        rm.insert("foo", BulkString::new("bar"));
        let res = rm.encode();
        println!("{:?}", String::from_utf8_lossy(&res).to_string());
        assert_eq!(res, b"%2\r\n+foo\r\n$3\r\nbar\r\n+hello\r\n$5\r\nworld\r\n");
    }

    #[test]
    fn test_resp_map_decode() {
        let mut data = BytesMut::from("%2\r\n+foo\r\n$3\r\nbar\r\n+hello\r\n$5\r\nworld\r\n");
        let res = RespMap::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(
            res.encode(),
            b"%2\r\n+foo\r\n$3\r\nbar\r\n+hello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_resp_map_decode_incomplete() {
        let mut data = BytesMut::from("%2\r\n+foo\r\n$3\r\nbar\r\n+hello\r\n$5\r\nwo");
        let err = RespMap::decode(&mut data).unwrap_err();
        assert_eq!(err, RespError::RespNotComplete);

        let mut data = BytesMut::from("%2\r\n+foo\r\n$3\r\nbar\r\n+hello\r\n$5\r");
        let err = RespMap::decode(&mut data).unwrap_err();
        assert_eq!(err, RespError::RespNotComplete);

        let mut data = BytesMut::from("%2\r\n+foo\r\n$3\r\nbar\r\nhello\r\n$5\r");
        let err = RespMap::decode(&mut data).unwrap_err();
        assert_ne!(err, RespError::RespNotComplete);
    }
}
