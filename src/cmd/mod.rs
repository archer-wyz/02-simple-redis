mod map;

use crate::{BulkString, RespArray, RespFrame};
use thiserror::Error;

pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
}

pub struct Get {
    pub key: String,
}

pub struct Set {
    pub key: String,
    pub value: RespFrame,
}

pub struct HGet {
    pub key: String,
    pub field: String,
}

pub struct HSet {
    pub key: String,
    pub field: String,
    pub value: RespFrame,
}

pub trait CommandExecutor {
    fn execute(&self) -> RespFrame;
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

// impl TryFrom<RespArray> for Command {
//     type Error = CommandError;
//
//     fn try_from(array: RespArray) -> Result<Self, Self::Error> {
//
//     }
// }

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
}
