use super::*;
use bytes::Buf;

// :[<+|->]<value>\r\n
impl RespEncode for i64 {
    fn encode(&self) -> Vec<u8> {
        let sign = if *self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}

impl RespDecode for i64 {
    const PREFIX: &'static str = ":";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (s, len) = split_cr_lf(data)?;
        let res =
            String::from_utf8_lossy(s)
                .parse::<i64>()
                .map_err(|_| RespError::RespParseError {
                    typ: "i64".to_string(),
                    data: String::from_utf8_lossy(s).to_string(),
                })?;
        data.advance(len);
        Ok(res.into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_i64_encode() {
        let i = 123;
        let res = i.encode();
        assert_eq!(res, b":+123\r\n");

        let i = -123;
        let res = i.encode();
        assert_eq!(res, b":-123\r\n");
    }

    #[test]
    fn test_i64_decode() {
        let mut data = BytesMut::from(":123\r\n");
        let res = i64::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b":+123\r\n");

        let mut data = BytesMut::from(":-123\r\n");
        let res = i64::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b":-123\r\n");
    }
}
