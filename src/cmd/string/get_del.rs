use bytes::Bytes;
use crate::db::SharedDb;
use crate::frame::{Frame, FrameError, FrameIter};
use crate::RedisResult;

pub(crate) struct GetDel(String);


impl GetDel {
    pub(crate) fn parse_frames(iter: &mut FrameIter) -> Result<Self, FrameError> {
        let key = iter.next_string()?;
        Ok(Self(key))
    }

    pub async fn execute(self, db: &SharedDb) -> RedisResult<Frame> {
        let mut store = db.lock();

        let data = store.remove(&self.0);
        if let Some(data) = data {
            return Ok(Frame::Bulk(data));
        } else {
            Ok(Frame::nil())
        }
    }
}
