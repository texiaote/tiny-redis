use bytes::Buf;

use crate::db::SharedDb;
use crate::frame::{Frame, FrameError, FrameIter};
use crate::RedisResult;

mod string;

use string::{Append, Del, Get, Set};

mod ping;
mod hash;
mod list;
mod set;
mod sorted_set;
mod pub_sub;
mod command;
mod key;
mod unknown;

use ping::Ping;
use crate::cmd::string::{DecrBy, MultiGet};
use crate::cmd::unknown::Unknown;
use crate::codec::RedisFrame;


pub(crate) trait Command {}

#[derive(Debug)]
pub(crate) enum Cmd {
    Set(Set),
    Get(Get),
    MGet(MultiGet),
    Del(Del),
    DecrBy(DecrBy),
    APPEND(Append),
    Ping(Ping),
    UnKnown(Unknown),
}

impl Cmd {
    pub(crate) async fn execute(self, db: &SharedDb) -> RedisResult<Frame> {
        match self {
            Cmd::Set(set) => set.execute(&db).await,
            Cmd::Get(get) => get.execute(&db).await,
            Cmd::MGet(multi_get) => multi_get.execute(&db).await,
            Cmd::DecrBy(decr_by) => decr_by.execute(&db).await,
            Cmd::Ping(ping) => ping.execute().await,
            Cmd::Del(del) => del.execute(&db).await,
            Cmd::APPEND(append) => append.execute(&db).await,
            Cmd::UnKnown(unknown) => unknown.execute().await,
        }
    }
}

impl TryFrom<Frame> for Cmd {
    type Error = FrameError;

    fn try_from(value: Frame) -> Result<Self, Self::Error> {
        let frames = match value {
            Frame::Array(array) => array,
            _ => return Err("request cmd must be array type".into())
        };

        let mut frame_iter = FrameIter::new(frames);

        let command = match frame_iter.next_string()?.to_uppercase().as_str() {
            "GET" =>
                Ok(Cmd::Get(Get::parse_frames(&mut frame_iter)?)),
            "MGET" => Ok(Cmd::MGet(MultiGet::parse_frames(&mut frame_iter)?)),
            "SET" => Ok(Cmd::Set(Set::parse_frames(&mut frame_iter)?)),
            "DEL" => Ok(Cmd::Del(Del::parse_frames(&mut frame_iter)?)),
            "APPEND" => Ok(Cmd::APPEND(Append::parse_frames(&mut frame_iter)?)),
            "PING" => Ok(Cmd::Ping(Ping)),
            "DECRBY" | "DECR" => Ok(Cmd::DecrBy(DecrBy::parse_frames(&mut frame_iter, true)?)),
            "INCRBY" | "INCR" => Ok(Cmd::DecrBy(DecrBy::parse_frames(&mut frame_iter, false)?)),
            other =>
                Ok(Cmd::UnKnown(Unknown::new(other.to_string()))),
        };

        frame_iter.finish()?;

        command
    }
}

impl TryFrom<Vec<RedisFrame>> for Cmd {
    type Error = std::io::Error;

    fn try_from(value: Vec<RedisFrame>) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use crate::cmd::Cmd;
    use crate::frame::Frame;

    #[tokio::test]
    async fn frame_to_command_test() {

        //构造frame,必须是命令的形式

        let mut vec = vec![];
        vec.push(Frame::Simple("SET".to_string()));
        vec.push(Frame::Simple("FOO".to_string()));
        vec.push(Frame::Simple("abcd".to_string()));

        let frame = Frame::Array(vec);

        let command: Cmd = frame.try_into().unwrap();

        println!();
    }
}