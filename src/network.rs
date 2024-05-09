use crate::{
    Backend, Command, CommandExecutor, RespDecode, RespEncode, RespError, RespFrame, SimpleError,
};
use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::info;

#[derive(Debug)]
struct RespCodec {}

#[derive(Debug)]
struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}

#[derive(Debug)]
struct RedisResponse {
    frame: RespFrame,
}

pub async fn stream_handler(stream: TcpStream, backend: Backend) -> Result<()> {
    let mut framed = Framed::new(stream, RespCodec::new());
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                let request = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                let response = request_handler(request).await?;
                info!("Sending response: {:?}", response.frame);
                framed.send(response.frame).await?;
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(()),
        }
    }
}

async fn request_handler(request: RedisRequest) -> Result<RedisResponse> {
    let (frame, backend) = (request.frame, request.backend);
    let cmd = match Command::try_from(frame) {
        Ok(c) => c,
        Err(e) => {
            return Ok(RedisResponse {
                frame: SimpleError::new(format!("Command Err: {}", e)).into(),
            })
        }
    };
    info!("Executing command: {:?}", cmd);
    let frame = cmd.execute(&backend);
    Ok(RedisResponse { frame })
}

impl Encoder<RespFrame> for RespCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: RespFrame, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let vec = item.encode();
        dst.extend_from_slice(&vec);
        Ok(())
    }
}

impl RespCodec {
    pub fn new() -> Self {
        RespCodec {}
    }
}

impl Decoder for RespCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;

    // make sure the bytes' cursor is at the ending after successfully decoding
    // TODO
    //  May be we don't need to read again when we get RespNotComplete.
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        //let mut src_clone = src.clone();
        match RespFrame::decode(src) {
            Ok(resp) => {
                info!("Received: {}", resp);
                Ok(Some(resp))
            }
            Err(e) => match e {
                RespError::RespNotComplete => Ok(None),
                _ => Err(anyhow!(format!("{:?}", e))),
            },
        }
    }
}
