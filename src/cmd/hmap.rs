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
            (Some(k), Some(f)) => Ok(HGet {
                key: k
                    .try_to_string()
                    .map_err(|e| CommandError::InvalidCommand(e.to_string()))?,
                field: f
                    .try_to_string()
                    .map_err(|e| CommandError::InvalidCommand(e.to_string()))?,
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
            (Some(k), Some(f), Some(v)) => Ok(HSet {
                key: k
                    .try_to_string()
                    .map_err(|e| CommandError::InvalidCommand(e.to_string()))?,
                field: f
                    .try_to_string()
                    .map_err(|e| CommandError::InvalidCommand(e.to_string()))?,
                value: v,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl TryFrom<RespArray> for HMGet {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let args = get_args_without_check(value, "hmget")?;
        let (key, fields) = parse_key_values_as_string(args)?;
        Ok(HMGet { key, fields })
    }
}

impl CommandExecutor for HMGet {
    fn execute(&self, backend: &Backend) -> RespFrame {
        let mut res = Vec::new();
        for field in &self.fields {
            res.push(
                backend
                    .hget(&self.key, field)
                    .unwrap_or_else(|| RespNull::new().into()),
            );
        }
        RespFrame::Array(RespArray::with_vec(res))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::RespDecode;
    use bytes::BytesMut;
    #[test]
    fn test_hmget() {
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(
            b"*4\r\n$5\r\nhmget\r\n$3\r\nkey\r\n$6\r\nfield1\r\n$6\r\nfield2\r\n",
        );
        let frame = RespFrame::decode(&mut bytes).unwrap();
        let hmget = Command::try_from(frame).unwrap();
        let b = Backend::default();
        b.hset("key".to_string(), "field1".to_string(), "value1".into());
        b.hset("key".to_string(), "field2".to_string(), "value2".into());

        let res = hmget.execute(&b);
        assert_eq!(
            res,
            RespFrame::Array(RespArray::with_vec(vec![
                SimpleString::new("value1").into(),
                SimpleString::new("value2").into()
            ]))
        );
    }
}
