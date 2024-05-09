use super::*;

impl CommandExecutor for Echo {
    fn execute(&self, _backend: &crate::Backend) -> RespFrame {
        self.value.clone()
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut args = get_args(value, "echo", 1)?.into_iter();
        match args.next() {
            Some(v) => Ok(Echo { value: v }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::RespDecode;
    use bytes::BytesMut;

    #[test]
    fn test_echo() {
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(b"*2\r\n$4\r\necho\r\n$5\r\nhello\r\n");
        let frame = RespFrame::decode(&mut bytes).unwrap();
        let echo = Command::try_from(frame).unwrap();
        let res = echo.execute(&crate::Backend::default());

        assert_eq!(res.try_to_string().unwrap(), "hello")
    }
}
