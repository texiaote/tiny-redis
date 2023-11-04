use bytes::Bytes;
use crate::db::SharedDb;
use crate::frame::{Frame, FrameError, FrameIter};
use crate::RedisResult;

#[derive(Debug)]
pub(crate) struct GetAndSet {
    key: String,
    value: Bytes,
}

impl GetAndSet {
    pub(crate) fn parse_frames(iter: &mut FrameIter) -> Result<Self, FrameError> {
        let key = iter.next_string()?;
        let value = iter.next_bytes()?;
        Ok(Self {
            key,
            value,
        })
    }

    pub(crate) async fn execute(self, db: &SharedDb) -> RedisResult<Frame> {
        let mut shared_db = db.lock();
        // if let Some(data) = shared_db.get_bytes(self.0.as_str()) {
        //     shared_db.update_bytes(self.key, self.value);
        //     return Ok(Frame::Bulk(data));
        // }
        Ok(Frame::nil())
    }
}