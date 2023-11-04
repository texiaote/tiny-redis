use std::io::ErrorKind;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

pub struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(i) = src.iter().position(|&b| b == b'\n') {
            let line = src.split_to(i);
            src.advance(1); // skip the '\n'
            match std::str::from_utf8(&line) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UTF-8")),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder<String> for LineCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put(item.as_bytes());
        dst.put_u8(b'\n');
        Ok(())
    }
}


pub struct RedisCodec;

pub enum RedisFrame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Bytes),
    Array(Vec<RedisFrame>),
}

impl Decoder for RedisCodec {
    type Item = RedisFrame;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if !src.has_remaining() {
            return Ok(None);
        }
        let frame = match src.get_u8() {
            b'+' => {
                RedisFrame::Simple(Self::get_line(src)?)
            }
            b'-' => {
                RedisFrame::Error(Self::get_line(src)?)
            }
            b':' => {
                RedisFrame::Integer(Self::get_decimal(src)?)
            }
            b'$' => {
                let len = Self::get_decimal(src)? as usize;

                let slice = Bytes::copy_from_slice(&src.chunk()[..len]);
                src.advance(len);

                if src.get_u8() != b'\r' || src.get_u8() != b'\n' {

                    // 抛出异常
                    return Err(std::io::Error::new(ErrorKind::InvalidData, ""));
                }
                RedisFrame::Bulk(slice)
            }
            b'*' => {
                let len = Self::get_decimal(src)? as usize;

                let mut vec = vec![];
                for _ in 0..len {
                    if let Ok(Some(frame)) = self.decode(src) {
                        vec.push(frame);
                    } else {
                        return Err(std::io::Error::new(ErrorKind::InvalidData, "error"));
                    }
                }

                RedisFrame::Array(vec)
            }
            other => {
                return Err(std::io::Error::new(ErrorKind::InvalidData, format!("redis command not find start with {} ", other)));
            }
        };
        Ok(Some(frame))
    }
}

impl Encoder<RedisFrame> for RedisCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: RedisFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            RedisFrame::Simple(simple) => {
                dst.put_u8(b'+');
                dst.put_slice(simple.as_bytes());
                dst.put_slice("\r\n".as_bytes());
            }
            RedisFrame::Error(error) => {
                dst.put_u8(b':');
                dst.put_slice(error.as_bytes());
                dst.put_slice("\r\n".as_bytes());
            }
            RedisFrame::Integer(integer) => {
                dst.put_u8(b':');
                dst.put_slice(integer.to_string().as_bytes());
                dst.put_slice("\r\n".as_bytes());
            }
            RedisFrame::Bulk(bytes) => {
                dst.put_u8(b'$');

                // 先获取字节数量
                let len = bytes.len();

                dst.put_slice(len.to_string().as_bytes());
                dst.put_slice("\r\n".as_bytes());

                // 将字节数组存入
                dst.put_slice(&bytes);
                dst.put_slice("\r\n".as_bytes());
            }
            RedisFrame::Array(array) => {
                dst.put_u8(b'*');
                // 先拿到数组的数量
                let len = array.len();
                dst.put_slice(len.to_string().as_bytes());
                dst.put_slice("\r\n".as_bytes());


                for frame in array {
                    self.encode(frame, dst)?;
                }
                dst.put_slice("\r\n".as_bytes());
            }
        }

        Ok(())
    }
}

impl RedisCodec {
    fn get_line(src: &mut BytesMut) -> Result<String, std::io::Error> {
        if let Some(i) = src.iter().position(|&b| b == b'\r') {

            // 将'\r'位置前面的数据都拿到
            let line = src.split_to(i);
            src.advance(1);

            //判断下一个数是不是'\n'
            if src.get_u8() != b'\n' {
                return Err(std::io::Error::new(ErrorKind::InvalidData, "data not end with \r\n"));
            }

            match String::from_utf8(line.to_vec()) {
                Ok(line) => Ok(line),
                Err(_) => Err(std::io::Error::new(ErrorKind::InvalidData, "Invalid UTF-8"))
            }
        } else {
            Err(std::io::Error::new(ErrorKind::InvalidInput, "Invalid input"))
        }
    }

    fn get_decimal(src: &mut BytesMut) -> Result<i64, std::io::Error> {
        let line = Self::get_line(src)?;

        atoi::atoi::<i64>(line.as_bytes()).ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "invalid frame format"))
    }
}

#[cfg(test)]
mod test {
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    use crate::codec::{RedisCodec, RedisFrame};

    #[test]
    fn encode_test() {
        let framea = RedisFrame::Integer(1234);
        let frameb = RedisFrame::Simple("abcd".to_string());

        let frame = RedisFrame::Array(vec![framea, frameb]);

        let mut stream = BytesMut::new();
        let mut codec = RedisCodec;
        let bytes = codec.encode(frame, &mut stream);

        let frame2 = codec.decode(&mut stream).unwrap().unwrap();
        println!();
    }
}
