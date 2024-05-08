use anyhow::Result;
use simple_redis::{stream_handler, Backend};
use tokio::net::TcpListener;
use tracing::{info, warn, Level};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE) // 设置日志级别
        .init();

    let addr = "0.0.0.0:6379";
    info!("Simple-Redis-Server is listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);
        tokio::spawn(async move {
            info!("Handling connection from: {}", raddr);
            match stream_handler(stream, Backend::new()).await {
                Ok(_) => info!("Connection from {} is closed", raddr),
                Err(e) => warn!("Error processing conn with {}: {:?}", raddr, e),
            }
        });
    }
}
