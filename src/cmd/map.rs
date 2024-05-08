use super::*;
use crate::{Backend, RespArray, RespNull};
use anyhow::Result;

impl TryFrom<RespArray> for Get {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut args = get_args(value, "get", 1)?.into_iter();
        match args.next() {
            Some(k) => Ok(Get {
                key: k
                    .try_to_string()
                    .map_err(|e| CommandError::InvalidCommand(e.to_string()))?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl CommandExecutor for Get {
    fn execute(&self, backend: &Backend) -> RespFrame {
        backend
            .get(&self.key)
            .unwrap_or_else(|| RespNull::new().into())
    }
}

impl TryFrom<RespArray> for Set {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut args = get_args(value, "set", 2)?.into_iter();
        match (args.next(), args.next()) {
            (Some(k), Some(v)) => Ok(Set {
                key: k
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

impl CommandExecutor for Set {
    fn execute(&self, backend: &Backend) -> RespFrame {
        backend.set(self.key.clone(), self.value.clone());
        RESP_OK.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_try_from() -> Result<()> {
        let mut ra = RespArray::new();
        ra.try_push("get")?;
        ra.try_push("key")?;
        let get = Get::try_from(ra)?;
        assert_eq!(get.key, "key");
        Ok(())
    }
}
