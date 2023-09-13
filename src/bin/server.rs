use tokio::net::TcpListener;
use tokio::signal;

use my_own_mini_redis::RedisResult;
use my_own_mini_redis::server::Server;

#[tokio::main]
pub async fn main() -> RedisResult<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let server = Server::new(listener);

    server.run().await?;


    Ok(())
}

