use std::time::Duration;
use bytes::Bytes;
use tokio::time::Instant;
use crate::db::SharedDb;
use crate::frame::{Frame, FrameError, FrameIter};
use crate::RedisResult;


/// https://redis.io/commands/set/
/// Syntax: SET key value [NX|XX] [GET] [EX seconds | PX milliseconds] | EXAT unix-time-seconds | PXAT unix-time-milliseconds | KEEPTTL ]
/// - EX seconds: Set the specified expire time, in seconds
/// - PX milliseconds: Set the specified expire time, in milliseconds
/// - EXAT timestamp-seconds: Set the specified Unix time at which the key will expire, in seconds
/// - PXAT timestamp-milliseconds: Set the specified Unix time at which the key will expire, in milliseconds
/// - NX: Only set the key if it does not already exist
/// - XX: Only set the key if it already exist
/// - KEEPTTL: Retain the time to live associated with the key
/// - GET: Return the old string stored at key, or nil if key did not exits. An error is returned and `SET` aborted if the value stored at key is not a string
#[derive(Debug)]
pub(crate) struct Set {
    key: String,
    value: Bytes,
    key_exists: bool,
    get: bool,
    expire_at: Option<Instant>,
}


impl Set {
    fn new(key: String, value: Bytes) -> Self {
        Self {
            key,
            value,
            key_exists: false,
            get: false,
            expire_at: None,
        }
    }
    pub(crate) fn parse_frames(iter: &mut FrameIter) -> Result<Self, FrameError> {

        // key值
        let key = iter.next_string()?;

        //value值
        let value = iter.next_bytes()?;

        let mut set = Self::new(key, value);

        while let Ok(keyword) = iter.next_string() {
            match keyword.to_uppercase().as_str() {
                "EX" => {
                    let secs = iter.next_int()?;

                    let expire_at = Instant::now() + Duration::from_secs(secs as u64);
                    set.expire_at = Some(expire_at);
                }
                "PX" => {
                    let milli_secs = iter.next_int()?;

                    let expire_at = Instant::now() + Duration::from_millis(milli_secs as u64);
                    set.expire_at = Some(expire_at);
                }
                "EXAT" => {
                    let expire_secs_timestamp = iter.next_int()? as u64;
                }
                "PXAT" => {
                    let expire_millisecs_timestamp = iter.next_int()? as u64;
                }
                "NX" => {
                    set.key_exists = false;
                }
                "XX" => {
                    set.key_exists = true;
                }
                "GET" => {
                    set.get = true;
                }

                "KEEPTTL" => {}
                others => {
                    return Err(format!("unhandle keyword:{}", others).into());
                }
            }
        }


        let expire = match iter.next_string() {
            Ok(s) if s.to_uppercase() == "EX" => {
                let secs = iter.next_int()?;
                Some(Duration::from_secs(secs as u64))
            }
            Ok(s) if s.to_uppercase() == "PX" => {
                let milli_secs = iter.next_int()?;
                Some(Duration::from_millis(milli_secs as u64))
            }
            Ok(_) => return Err("currently `SET` only supports the expiration option".into()),
            Err(FrameError::EndOfStream) => None,
            Err(e) => return Err(e.into())
        };

        Ok(set)
    }
    pub async fn execute(self, db: &SharedDb) -> RedisResult<Frame> {
        let mut shared = db.lock();


        shared.set_bytes(self.key, self.value, None);

        Ok(Frame::ok())
    }
}


#[cfg(test)]
mod test {
    use bytes::Bytes;
    use tokio::sync::broadcast;
    use crate::cmd::string::Set;
    use crate::db::{Db, SharedDb};

    fn init_db() -> SharedDb {
        let (sender, _) = broadcast::channel(1);
        Db::new(sender.subscribe())
    }

    #[tokio::test]
    async fn set_test() {
        let db = init_db();
    }
}