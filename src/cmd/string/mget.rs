use crate::db::SharedDb;
use crate::frame::{Frame, FrameError, FrameIter};
use crate::RedisResult;

#[derive(Debug)]
pub(crate) struct MultiGet {
    keys: Vec<String>,
}

impl MultiGet {
    pub(crate) fn parse_frames(iter: &mut FrameIter) -> Result<Self, FrameError> {
        let mut keys = vec![];

        while let Ok(key) = iter.next_string() {
            keys.push(key);
        }

        Ok(Self {
            keys
        })
    }

    pub(crate) async fn execute(self, db: &SharedDb) -> RedisResult<Frame> {
        let store = db.lock();

        let mut vec = vec![];
        for key in self.keys {
            if let Some(data) = store.get_bytes(key) {
                vec.push(Frame::Bulk(data));
            } else {
                vec.push(Frame::nil());
            }
        }
        Ok(Frame::Array(vec))
    }
}
