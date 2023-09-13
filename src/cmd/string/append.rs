use bytes::{Bytes, BytesMut};
use crate::db::{Db, SharedDb};
use crate::frame::{Frame, FrameError, FrameIter};
use crate::RedisResult;

#[derive(Debug)]
pub(crate) struct Append {
    key: String,
    value: Bytes,
}

impl Append {
    pub(crate) fn parse_frames(iter: &mut FrameIter) -> Result<Self, FrameError> {
        let key = iter.next_string()?;
        let value = iter.next_bytes()?;

        Ok(Append {
            key,
            value,
        })
    }

    pub(crate) async fn execute(self, db: &SharedDb) -> RedisResult<Frame> {
        let mut store = db.lock();
        if let Some(data) = store.get_bytes(self.key.clone()) {

            // 两个拼接
            let mut merged_data = BytesMut::new();

            merged_data.extend_from_slice(&data[..]);
            merged_data.extend_from_slice(&self.value[..]);

            let data = merged_data.freeze();
            store.update_bytes(&self.key, data);
        }
        Ok(Frame::ok())
    }
}