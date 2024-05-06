use super::*;
use bytes::BytesMut;
use enum_dispatch::enum_dispatch;
use thiserror::Error;

#[enum_dispatch(RespEncode)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum RespFrame {
    SimpleString(SimpleString),
    Error(SimpleError),
    Integer(i64),
    BulkString(BulkString),
    Array(RespArray),
    Null(RespNull),
    NullArray(RespNullArray),
    Boolean(bool),
    Double(f64),
    Map(RespMap),
    Set(RespSet),
}

impl RespDecode for RespFrame {
    const PREFIX: &'static str = "";
    fn decode(data: &mut BytesMut) -> Result<Self, RespError> {
        if data.is_empty() {
            return Err(crate::resp::RespError::RespIsEmpty);
        }
        let prefix = data[0] as char;
        match prefix {
            '+' => Ok(SimpleString::decode(data)?.into()),
            '*' => Ok(RespArray::decode(data).map_or()),
            '$' => Ok(BulkString::decode(data)?.into()),
            '#' => Ok(bool::decode(data)?.into()),
            '-' => Ok(SimpleError::decode(data)?.into()),
            ':' => Ok(i64::decode(data)?.into()),
            ',' => Ok(f64::decode(data)?.into()),
            _ => Err(RespError::RespNotSupport(prefix)),
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum RespError {
    #[error("Invalid RESP frame {0}")]
    RespNotComplete(String),
    #[error("Empty RESP frame")]
    RespIsEmpty,
    #[error("Prefix not support {0}")]
    RespNotSupport(char),
    #[error("RESP length {expected:?} not equals to decoded length {decoded:?}")]
    RespNotEqualLength { expected: usize, decoded: usize },
    #[error("RESP Parse Error: try parse (data: {data:?}) into (type: {typ:?})")]
    RespParseError { typ: String, data: String },
    #[error("RESP (type: {typ:?}) Wrapped (err: {err:?})")]
    RespWrappedError { typ: String, err: Box<RespError> },
}

impl<const N: usize> From<&[u8; N]> for RespFrame {
    fn from(s: &[u8; N]) -> Self {
        RespFrame::BulkString(BulkString::new(s.to_vec()))
    }
}
