use crate::db::Db;
use crate::frame::{Frame, FrameError, FrameIter};
use crate::RedisResult;

#[derive(Debug)]
pub(crate) struct Del(Vec<String>);


impl Del {
    pub(crate) fn parse_frames(iter: &mut FrameIter) -> Result<Self, FrameError> {
        let mut vec = vec![];
        while let Ok(key) = iter.next_string() {
            vec.push(key);
        }

        Ok(Self(vec))
    }

    pub(crate) async fn execute(self, db: &Db) -> RedisResult<Frame> {
        let mut store = db.lock();
        store.remove_vec(&self.0);
        Ok(Frame::Simple("OK".to_string()))
    }
}