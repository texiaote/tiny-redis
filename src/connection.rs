use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;

use crate::frame::Frame;
use crate::RedisResult;

pub(crate) struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub(crate) fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4 * 1024),
        }
    }


    pub(crate) async fn read_frame(&mut self) -> RedisResult<Option<Frame>> {
        loop {
            if let Ok(frame) = Frame::parse_protocol(self.buffer.chunk()) {
                self.buffer.clear();
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if (self.buffer.is_empty()) {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            }
        }
    }

    pub(crate) async fn write_frame(&mut self, frame: Frame) -> RedisResult<()> {
        let bytes = frame.to_protocol()?;

        self.stream.write_all(&bytes[..]).await?;
        //刷新数据
        self.stream.flush().await?;

        Ok(())
    }
}