use crate::frame::Frame;
use crate::RedisResult;

#[derive(Debug)]
pub(crate) struct Unknown {
    cmd: String,
}

impl Unknown {
    pub(crate) fn new(cmd: String) -> Self {
        Self {
            cmd
        }
    }
    pub(crate) async fn execute(self) -> RedisResult<Frame> {
        Ok(Frame::Error("Unknown Command".to_string()))
    }
}