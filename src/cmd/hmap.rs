use super::*;
use crate::{Backend, CommandExecutor, RespFrame, RespNull};

impl CommandExecutor for HGet {
    fn execute(&self, backend: &Backend) -> RespFrame {
        backend
            .hget(&self.key, &self.field)
            .unwrap_or_else(|| RespNull::new().into())
    }
}

impl CommandExecutor for HSet {
    fn execute(&self, backend: &Backend) -> RespFrame {
        backend.hset(self.key.clone(), self.field.clone(), self.value.clone());
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut args = get_args(value, "hget", 2)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(k)), Some(RespFrame::BulkString(f))) => Ok(HGet {
                key: k.to_string(),
                field: f.to_string(),
            }),
            (Some(RespFrame::SimpleString(k)), Some(RespFrame::SimpleString(f))) => Ok(HGet {
                key: k.to_string(),
                field: f.to_string(),
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HSet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut args = get_args(value, "hset", 3)?.into_iter();
        match (args.next(), args.next(), args.next()) {
            (Some(RespFrame::BulkString(k)), Some(RespFrame::BulkString(f)), Some(v)) => Ok(HSet {
                key: k.to_string(),
                field: f.to_string(),
                value: v,
            }),
            (Some(RespFrame::SimpleString(k)), Some(RespFrame::SimpleString(f)), Some(v)) => {
                Ok(HSet {
                    key: k.to_string(),
                    field: f.to_string(),
                    value: v,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}
