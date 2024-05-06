mod array;
mod bool;
mod bulk_string;
mod double;
mod frame;
mod i64;
mod map;
mod null;
mod set;
mod simple_error;
mod simple_string;

use crate::resp::RespFrame::Double;
use bytes::BytesMut;
use core::f64;
use enum_dispatch::enum_dispatch;
use std::collections::BTreeMap;

pub const DEFAULT_CAPACITY: usize = 32;

pub use self::{
    array::RespArray,
    bulk_string::BulkString,
    frame::{RespError, RespFrame},
    map::RespMap,
    null::{RespNull, RespNullArray},
    set::RespSet,
    simple_error::SimpleError,
    simple_string::SimpleString,
};

#[enum_dispatch]
pub trait RespEncode {
    fn encode(&self) -> Vec<u8>;
}

// RespDecode trait
//
// The trait is used to decode RESP frame from BytesMut,
// whose length will be zero if successfully decoded.
pub trait RespDecode: Sized {
    const PREFIX: &'static str;
    // TODO
    // reactor the signature of decode method from data:&mut BytesMut to mut data: BytesMut
    fn decode(data: &mut BytesMut) -> Result<Self, RespError>;
}

fn split_cr_lf(data: &BytesMut) -> Result<(&[u8], usize), RespError> {
    let mut pos = 0;
    while pos < data.len() {
        if data[pos] == b'\r' {
            if pos + 1 < data.len() && data[pos + 1] == b'\n' {
                return Ok((&data[..pos], pos + 2));
            }
        }
        pos += 1;
    }
    Err(RespError::RespNotComplete("CRLF not found".to_string()))
}

fn parse_length(data: &BytesMut) -> Result<(usize, usize), RespError> {
    let (s, pos) = split_cr_lf(data)?;
    let s = String::from_utf8_lossy(s);
    let len = s
        .parse()
        .map_err(|_| RespError::RespNotComplete("Invalid length".to_string()))?;
    Ok((len, pos))
}
