use bytes::Bytes;
use crate::frame::Frame;
use crate::RedisResult;

#[derive(Debug)]
pub(crate) struct Ping;

impl Ping {
    pub(crate) async fn execute(self) -> RedisResult<Frame> {
        Ok(Frame::Bulk(Bytes::from("pong".as_bytes())))
    }
}