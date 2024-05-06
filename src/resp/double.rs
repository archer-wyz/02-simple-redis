use super::*;
use bytes::Buf;

impl RespEncode for f64 {
    fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(DEFAULT_CAPACITY);
        buf.extend_from_slice(b",");
        let res = if self.abs() > 1e-6 {
            format!("{:?}\r\n", self)
        } else {
            format!("{:e}\r\n", self)
        };
        buf.extend_from_slice(res.as_bytes());
        buf
    }
}

impl RespDecode for f64 {
    const PREFIX: &'static str = ",";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        data.advance(Self::PREFIX.len());
        let (s, len) = split_cr_lf(data)?;
        let res =
            String::from_utf8_lossy(s)
                .parse::<f64>()
                .map_err(|_| RespError::RespParseError {
                    typ: "double".to_string(),
                    data: String::from_utf8_lossy(s).to_string(),
                })?;
        data.advance(len);
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_double_encode() {
        let d = 1.0;
        let res = d.encode();
        assert_eq!(
            String::from_utf8(res).unwrap(),
            String::from_utf8(b",1.0\r\n".to_vec()).unwrap()
        );
        let d2 = -1.0;
        let res2 = d2.encode();
        assert_eq!(res2, b",-1.0\r\n");
        let d3 = 1.0e-7;
        let res3 = d3.encode();
        assert_eq!(res3, b",1e-7\r\n");

        let d3 = -2.3e-8;
        let res3 = d3.encode();
        assert_eq!(res3, b",-2.3e-8\r\n");
    }

    #[test]
    fn test_double_decode() {
        let mut data = BytesMut::from(",1.0\r\n");
        let res = f64::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b",1.0\r\n");

        let mut data = BytesMut::from(",-1.0\r\n");
        let res = f64::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b",-1.0\r\n");

        let mut data = BytesMut::from(",1e-7\r\n");
        let res = f64::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b",1e-7\r\n");

        let mut data = BytesMut::from(",-2.3e-8\r\n");
        let res = f64::decode(&mut data).unwrap();
        assert_eq!(data.len(), 0);
        assert_eq!(res.encode(), b",-2.3e-8\r\n");

        let mut data = BytesMut::from(",-asdf\r\n");
        let res = f64::decode(&mut data).unwrap_err();
        println!("{:?}", res);
        assert_eq!(
            res,
            RespError::RespParseError {
                typ: "double".to_string(),
                data: "-asdf".to_string(),
            }
        );
    }
}
