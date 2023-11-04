use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use my_own_mini_redis::codec::{LineCodec, RedisCodec, RedisFrame};
use futures::SinkExt;

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:6379").await.unwrap();

    let mut frame = Framed::new(stream, RedisCodec);
    frame.send(RedisFrame::Integer(1)).await.unwrap();
    //  frame.send("+abcd".to_string()).await.unwrap();
}