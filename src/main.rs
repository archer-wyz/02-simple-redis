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
    let backend = Backend::new();
    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr);
        let cloned_backend = backend.clone();
        tokio::spawn(async move {
            info!("Handling connection from: {}", raddr);
            // let cloned_backend = backend.clone();
            // EXISTS BUG!!!!!!
            // 21 | |             let cloned_backend = backend.clone();
            //    | |                                  ------- use occurs due to use in coroutine
            // 22 | |             match stream_handler(stream, cloned_backend).await {
            // ...  |
            // 25 | |             }
            // 26 | |         });
            //    | |_________^ value moved here, in previous iteration of loop
            match stream_handler(stream, cloned_backend).await {
                Ok(_) => info!("Connection from {} is closed", raddr),
                Err(e) => warn!("Error processing conn with {}: {:?}", raddr, e),
            }
        });
    }
}
