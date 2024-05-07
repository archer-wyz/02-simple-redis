use super::*;
use crate::RespArray;
use anyhow::Result;

impl TryFrom<RespArray> for Get {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut args = get_args(value, "get", 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(s)) => Ok(Get { key: s.to_string() }),
            Some(RespFrame::SimpleString(s)) => Ok(Get { key: s.to_string() }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
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