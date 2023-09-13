use bytes::Bytes;
use crate::db::SharedDb;
use crate::frame::{Frame, FrameError, FrameIter};
use crate::RedisResult;

#[derive(Debug)]
pub(crate) struct DecrBy {
    key: String,
    decrement: i64,
}

impl DecrBy {
    pub(crate) fn parse_frames(iter: &mut FrameIter, positive: bool) -> Result<Self, FrameError> {
        let key = iter.next_string()?;

        let mut decrement = 1;
        if (iter.has_remaining()) {
            decrement = iter.next_int()?;
        }
        if !positive {
            decrement = -decrement;
        }
        Ok(Self {
            key,
            decrement,
        })
    }

    pub(crate) async fn execute(self, db: &SharedDb) -> RedisResult<Frame> {
        let mut store = db.lock();

        if let Some(data) = store.get_bytes(&self.key) {
            let mut number = atoi::atoi::<i64>(&data).ok_or(0).unwrap();
            number -= self.decrement;

            // 转换成int操作后，更新
            store.update_bytes(&self.key, Bytes::from(number.to_string()));
            return Ok(Frame::Integer(number));
        } else {
            let mut number = 0;
            number -= self.decrement;
            //新建一个，
            store.set_bytes(&self.key, Bytes::from(number.to_string()), None);

            return Ok(Frame::Integer(number));
        }
    }
}