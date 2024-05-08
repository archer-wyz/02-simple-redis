mod hmap;
mod map;

use crate::{BulkString, RespArray, RespFrame, SimpleString};
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

    // unrecognized command
    Unrecognized(Unrecognized),
}

#[derive(Debug)]
pub struct Unrecognized;

#[derive(Debug)]
pub struct Get {
    pub key: String,
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
                    Some(RespFrame::BulkString(ref cmd)) => match cmd {
                        BulkString::Vec(v) => Ok(v.as_ref()),
                        _ => Err(CommandError::InvalidCommand(
                            "command should be bulk_string".to_string(),
                        )),
                    },
                    Some(RespFrame::SimpleString(ref cmd)) => Ok(cmd.0.as_bytes()),
                    _ => Err(CommandError::InvalidCommand(
                        "command should be string".to_string(),
                    )),
                }?;

                match v {
                    b"get" => Ok(Get::try_from(array)?.into()),
                    b"set" => Ok(Set::try_from(array)?.into()),
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

fn get_args(value: RespArray, command: &str, args: usize) -> Result<Vec<RespFrame>, CommandError> {
    let frame = match value.0 {
        None => return Err(CommandError::InvalidCommand("Empty command".to_string())),
        Some(v) => {
            if v.is_empty() {
                return Err(CommandError::InvalidCommand("Empty command".to_string()));
            }
            v
        }
    };

    match &frame[0] {
        RespFrame::BulkString(ref bs) => match bs {
            BulkString::Vec(v) => {
                if v.as_slice() != command.as_bytes() {
                    return Err(CommandError::NotEqualCommand);
                }
                if frame.len() != args + 1 {
                    return Err(CommandError::InvalidCommand(format!(
                        "Command args not equal, expect: {}, got: {}",
                        args,
                        frame.len() - 1
                    )));
                }
                Ok(frame.into_iter().skip(1).collect())
            }
            BulkString::Null => Err(CommandError::InvalidCommand(
                "Command type is bulk_string_null".to_string(),
            )),
        },
        RespFrame::SimpleString(ref ss) => {
            if ss.as_bytes() != command.as_bytes() {
                return Err(CommandError::NotEqualCommand);
            }
            if frame.len() != args + 1 {
                return Err(CommandError::InvalidCommand(format!(
                    "Command args not equal, expect: {}, got: {}",
                    args,
                    frame.len() - 1
                )));
            }
            Ok(frame.into_iter().skip(1).collect())
        }
        _ => Err(CommandError::InvalidCommand(
            "Command type should be simple_string or bulk_string".to_string(),
        )),
    }
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
            Command::Unrecognized(_) => {}
            _ => panic!("Command type not equal"),
        }
    }
}
