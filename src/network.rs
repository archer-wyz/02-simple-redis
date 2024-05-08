use crate::{Backend, RespDecode, RespEncode, RespError, RespFrame};
use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::info;

struct RespCodec {}

pub async fn stream_handler(stream: TcpStream, backend: Backend) -> Result<()> {
    let mut framed = Framed::new(stream, RespCodec::new());
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);
                let frame = backend.handle(frame);
                framed.send(frame).await?;
            }
            Some(Err(e)) => return Err(e),
            None => return Ok(()),
        }
    }
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
