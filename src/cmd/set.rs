use super::*;
use crate::Backend;

impl TryFrom<RespArray> for SADD {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let args = get_args_without_check(value, "sadd")?;
        let (key, members) = parse_key_values_as_string(args)?;
        Ok(SADD { key, members })
    }
}

impl CommandExecutor for SADD {
    fn execute(&self, backend: &Backend) -> RespFrame {
        let mut res = 0;
        for member in &self.members {
            res += backend.sadd(self.key.clone(), member.clone()) as i64;
        }
        res.into()
    }
}

impl TryFrom<RespArray> for SISMEBER {
    type Error = CommandError;

    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        let mut args = get_args(value, "sismember", 2)?.into_iter();
        match (args.next(), args.next()) {
            (Some(k), Some(m)) => Ok(SISMEBER {
                key: k
                    .try_to_string()
                    .map_err(|e| CommandError::InvalidCommand(e.to_string()))?,
                member: m
                    .try_to_string()
                    .map_err(|e| CommandError::InvalidCommand(e.to_string()))?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid argument".to_string(),
            )),
        }
    }
}

impl CommandExecutor for SISMEBER {
    fn execute(&self, backend: &Backend) -> RespFrame {
        match backend.sismember(&self.key, &self.member) {
            true => 1,
            false => 0,
        }
        .into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sadd() {
        let backend = Backend::default();
        backend.sadd("key".to_string(), "member1".to_string());
        let sadd = SADD {
            key: "key".to_string(),
            members: vec![
                "member1".to_string(),
                "member2".to_string(),
                "member3".to_string(),
            ],
        };
        let res = sadd.execute(&backend);
        assert_eq!(res.try_to_int().unwrap(), 2);
    }

    #[test]
    fn test_sismember() {
        let backend = Backend::default();
        backend.sadd("key".to_string(), "member1".to_string());
        let sismember = SISMEBER {
            key: "key".to_string(),
            member: "member1".to_string(),
        };
        let res = sismember.execute(&backend);
        assert_eq!(res.try_to_int().unwrap(), 1);
    }
}
