mod echo;
mod hmap;
mod map;
mod set;

use crate::{RespArray, RespFrame, SimpleString};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

// you could also use once_cell instead of lazy_static
lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HMGet(HMGet),

    SADD(SADD),
    SISMEBER(SISMEBER),

    Echo(Echo),

    // unrecognized command
    Unrecognized(Unrecognized),
}

#[derive(Debug)]
pub struct Unrecognized;

#[derive(Debug)]
pub struct SADD {
    pub key: String,
    pub members: Vec<String>,
}

#[derive(Debug)]
pub struct SISMEBER {
    pub key: String,
    pub member: String,
}

#[derive(Debug)]
pub struct Get {
    pub key: String,
}

#[derive(Debug)]
pub struct Echo {
    pub value: RespFrame,
}

#[derive(Debug)]
pub struct HMGet {
    pub key: String,
    pub fields: Vec<String>,
}

#[derive(Debug)]
pub struct Set {
    pub key: String,
    pub value: RespFrame,
}

#[derive(Debug)]
pub struct HGet {
    pub key: String,
    pub field: String,
}

#[derive(Debug)]
pub struct HSet {
    pub key: String,
    pub field: String,
    pub value: RespFrame,
}

#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(&self, backend: &crate::Backend) -> RespFrame;
}

#[derive(Debug, Error, PartialEq, PartialOrd)]
pub enum CommandError {
    // NotEqualCommand,
    // May try to use other command.
    #[error("Command type not equal")]
    NotEqualCommand,
    #[error("Unexpected error")]
    UnexpectedError,
    // InvalidCommand
    // It must have some error that must stop.
    #[error("Invalid command {0}")]
    InvalidCommand(String),
    #[error("Invalid type {0}")]
    InvalidArgument(String),
}

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;

    fn try_from(frame: RespFrame) -> Result<Self, Self::Error> {
        match frame {
            RespFrame::Array(array) => Command::try_from(array),
            _ => Err(CommandError::InvalidCommand("Invalid command".to_string())),
        }
    }
}

impl TryFrom<RespArray> for Command {
    type Error = CommandError;

    fn try_from(array: RespArray) -> Result<Self, Self::Error> {
        match &array.0 {
            Some(v) => {
                let v = match v.first() {
                    None => Err(CommandError::InvalidCommand("empty command".to_string())),
                    Some(v) => Ok(v.try_to_string().unwrap_or_default()),
                }?
                .to_lowercase();

                match v.as_bytes() {
                    b"get" => Ok(Get::try_from(array)?.into()),
                    b"set" => Ok(Set::try_from(array)?.into()),
                    b"hset" => Ok(HSet::try_from(array)?.into()),
                    b"hget" => Ok(HGet::try_from(array)?.into()),
                    b"echo" => Ok(Echo::try_from(array)?.into()),
                    b"hmget" => Ok(HMGet::try_from(array)?.into()),
                    b"sadd" => Ok(SADD::try_from(array)?.into()),
                    b"sismember" => Ok(SISMEBER::try_from(array)?.into()),
                    _ => Ok(Unrecognized.into()),
                }
            }
            None => Err(CommandError::InvalidCommand("Empty command".to_string())),
        }
    }
}

impl CommandExecutor for Unrecognized {
    fn execute(&self, _: &crate::Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

fn get_args_without_check(value: RespArray, command: &str) -> Result<Vec<RespFrame>, CommandError> {
    let frame = match value.0 {
        None => return Err(CommandError::InvalidCommand("Empty command".to_string())),
        Some(v) => v,
    };
    let mut iter = frame.into_iter();
    match iter.next() {
        None => Err(CommandError::InvalidCommand("Empty command".to_string())),
        Some(v) => match v.try_to_string() {
            Ok(v) => {
                if v.to_lowercase() != command {
                    return Err(CommandError::NotEqualCommand);
                }
                Ok(iter.collect())
            }
            Err(_) => Err(CommandError::InvalidCommand("Invalid command".to_string())),
        },
    }
}

fn get_args(value: RespArray, command: &str, args: usize) -> Result<Vec<RespFrame>, CommandError> {
    let frame = get_args_without_check(value, command)?;
    if frame.len() != args {
        return Err(CommandError::InvalidCommand(format!(
            "Command args not equal, expect: {}, got: {}",
            args,
            frame.len()
        )));
    }
    Ok(frame)
}

fn parse_key_values_as_string(args: Vec<RespFrame>) -> Result<(String, Vec<String>), CommandError> {
    let mut args = args.into_iter();
    let key = match args.next() {
        None => Err(CommandError::InvalidCommand("Empty command".to_string())),
        Some(v) => match v.try_to_string() {
            Ok(v) => Ok(v),
            Err(e) => Err(CommandError::InvalidCommand(e.to_string())),
        },
    }?;
    let fields: Vec<String> = args.filter_map(|v| v.try_to_string().ok()).collect();
    Ok((key, fields))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{RespArray, RespDecode};
    use bytes::BytesMut;

    #[test]
    fn test_get_args() {
        let mut data = BytesMut::from("*2\r\n$3\r\nget\r\n$3\r\nkey\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        let args = get_args(res, "get", 1).unwrap();
        assert_eq!(args.len(), 1);

        let mut data = BytesMut::from("*2\r\n+get\r\n$3\r\nkey\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        let args = get_args(res, "get", 1).unwrap();
        assert_eq!(args.len(), 1);

        let mut data = BytesMut::from("*3\r\n$3\r\nget\r\n$3\r\nkey\r\n+abc\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        let err = get_args(res, "get", 1).unwrap_err();
        assert_eq!(
            err,
            CommandError::InvalidCommand(format!(
                "Command args not equal, expect: {}, got: {}",
                1, 2
            ))
        );
    }

    #[test]
    fn test_try_from() {
        let mut data = BytesMut::from("*2\r\n$3\r\nget\r\n$3\r\nkey\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        let cmd = Command::try_from(res).unwrap();
        match cmd {
            Command::Get(_) => {}
            _ => panic!("Command type not equal"),
        }

        let mut data = BytesMut::from("*3\r\n$3\r\nset\r\n$3\r\nkey\r\n$5\r\nvalue\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        let cmd = Command::try_from(res).unwrap();
        match cmd {
            Command::Set(_) => {}
            _ => panic!("Command type not equal"),
        }

        let mut data = BytesMut::from("*3\r\n$4\r\nhget\r\n$3\r\nabc\r\n$3\r\nkey\r\n");
        let res = RespArray::decode(&mut data).unwrap();
        let cmd = Command::try_from(res).unwrap();
        match cmd {
            Command::HGet(_) => {}
            _ => panic!("Command type not equal"),
        }
    }
}
