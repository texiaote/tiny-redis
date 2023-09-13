use crate::frame::Frame;
use crate::RedisResult;


#[derive(Debug)]
pub(crate) struct Command;

impl Command {
    pub(crate) fn execute(self) -> RedisResult<Frame> {
        let list = vec![];


        Ok(Frame::Array(list))
    }
}