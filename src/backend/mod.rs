use crate::RespFrame;
use crate::SimpleString;
use tracing::info;

pub struct Backend {}

impl Backend {
    pub fn handle(&self, req: RespFrame) -> RespFrame {
        info!("Request: {:?}", req);
        SimpleString::new("+OK\r\n").into()
    }
}

impl Backend {
    pub fn new() -> Self {
        Backend {}
    }
}

impl Default for Backend {
    fn default() -> Self {
        Backend::new()
    }
}
