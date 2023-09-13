use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::io::Cursor;
use std::string::FromUtf8Error;
use std::vec;

use bytes::{Buf, BufMut, Bytes};

#[derive(Debug)]
pub(crate) enum Frame {
    Simple(String),
    Error(String),
    Integer(i64),
    Bulk(Bytes),
    Array(Vec<Frame>),
}

#[derive(Debug, PartialEq)]
pub enum FrameError {
    //没有完成
    Incomplete,

    EndOfStream,
}

#[derive(Debug)]
pub(crate) struct FrameIter(VecDeque<Frame>);


impl Frame {

    //从二进制协议中解析数据成Frame




    pub(crate) fn parse_protocol(buf: &[u8]) -> Result<Frame, FrameError> {
        let mut cursor = Cursor::new(buf);

        check(&mut cursor)?;

        // 从0开始读取
        cursor.set_position(0);
        // 正式的parse数据

        parse(&mut cursor)
    }


    // 将Frame转换成二进制协议
    pub(crate) fn to_protocol(self) -> Result<Bytes, FrameError> {
        let mut vec = vec![];
        match self {
            Frame::Simple(value) => {
                vec.push(b'+');
                vec.put_slice(value.as_bytes());
                vec.push(b'\r');
                vec.push(b'\n');
            }
            Frame::Error(error) => {
                vec.push(b'-');
                vec.put_slice(error.as_bytes());
                vec.push(b'\r');
                vec.push(b'\n');
            }
            Frame::Integer(value) => {
                vec.push(b':');

                let value_string = value.to_string();
                vec.put_slice(value_string.as_bytes());
                vec.push(b'\r');
                vec.push(b'\n');
            }
            Frame::Bulk(data) => {
                vec.push(b'$');
                let len = data.len();


                let len_string = len.to_string();

                vec.put_slice(len_string.as_bytes());
                vec.push(b'\r');
                vec.push(b'\n');

                vec.put_slice(&data);
                vec.push(b'\r');
                vec.push(b'\n');
            }
            Frame::Array(array) => {
                vec.push(b'*');
                let len = array.len();

                let len_string = len.to_string();

                vec.put_slice(len_string.as_bytes());
                vec.push(b'\r');
                vec.push(b'\n');

                for frame in array {
                    let protocol = frame.to_protocol()?;
                    vec.put_slice(protocol.chunk());
                }
            }
        }

        //这个末尾都要统一加的

        Ok(Bytes::from(vec))
    }
}

impl Frame {
    pub(crate) fn ok() -> Frame {
        Frame::Simple("OK".to_string())
    }
    pub(crate) fn nil() -> Frame {
        Frame::Integer(-1)
    }
}

impl FrameIter {
    pub(crate) fn new(frames: Vec<Frame>) -> Self {
        let mut vec_deque: VecDeque<Frame> = VecDeque::new();
        vec_deque.extend(frames);


        // Self(frames.into_iter())
        Self(vec_deque)
    }

    fn next(&mut self) -> Result<Frame, FrameError> {
        if let Some(frame) = self.0.pop_front() {
            return Ok(frame);
        } else {
            return Err(FrameError::EndOfStream);
        }
    }

    pub(crate) fn next_string(&mut self) -> Result<String, FrameError> {
        match self.next()? {
            Frame::Simple(simple) =>
                Ok(simple),
            Frame::Bulk(data) => {
                std::str::from_utf8(&data[..]).map(|s| s.to_string()).map_err(|_| "protocol error; invalid string".into())
            }
            frame => Err(format!("protocol error; expected simple or bulk frame, got {:?}", frame).into())
        }
    }

    pub(crate) fn next_bytes(&mut self) -> Result<Bytes, FrameError> {
        match self.next()? {
            Frame::Bulk(data) => Ok(data),
            Frame::Simple(s) => Ok(Bytes::from(s.into_bytes())),
            frame => Err(format!("protocol error; expected simple frame or bulk frame, got {:?}",
                                 frame).into())
        }
    }
    pub(crate) fn next_int(&mut self) -> Result<i64, FrameError> {
        use atoi::atoi;

        const MSG: &str = "protocol error; invalid number";

        match self.next()? {
            // An integer frame type is already stored as an integer.
            Frame::Integer(v) => Ok(v),
            // Simple and bulk frames must be parsed as integers. If the parsing
            // fails, an error is returned.
            Frame::Simple(data) => atoi::<i64>(data.as_bytes()).ok_or_else(|| MSG.into()),
            Frame::Bulk(data) => atoi::<i64>(&data).ok_or_else(|| MSG.into()),
            frame => Err(format!("protocol error; expected int frame but got {:?}", frame).into()),
        }
    }

    pub(crate) fn finish(&mut self) -> Result<(), FrameError> {
        if !self.has_remaining() {
            Ok(())
        } else {
            Err("protocol error; expected end of frame, but there was more".into())
        }
    }

    pub(crate) fn has_remaining(&self) -> bool {
        !self.0.is_empty()
    }
}

fn check(cursor: &mut Cursor<&[u8]>) -> Result<(), FrameError> {
    if !cursor.has_remaining() {
        return Err(FrameError::Incomplete);
    }

    match cursor.get_u8() {
        b'+' | b'-' => {
            let _ = get_line(cursor)?;
            return Ok(());
        }
        b':' => {
            let _ = get_decimal(cursor)?;
            return Ok(());
        }
        b'$' => {

            //先获得数量
            let n = get_decimal(cursor)? as usize;

            //然后忽略字符
            let _ = skip(cursor, n + 2)?;
            return Ok(());
        }
        b'*' => {
            let n = get_decimal(cursor)? as usize;

            for _ in 0..n {
                check(cursor)?;
            }
            return Ok(());
        }
        actual => { Err(format!("protocal error: invalid frame type bytes `{}`", actual).into()) }
    }
}


fn parse(cursor: &mut Cursor<&[u8]>) -> Result<Frame, FrameError> {
    match get_u8(cursor)? {
        b'+' => {
            Ok(Frame::Simple(get_line(cursor)?))
        }
        b'-' => {
            Ok(Frame::Error(get_line(cursor)?))
        }
        b':' => {
            Ok(Frame::Integer(get_decimal(cursor)?))
        }
        b'$' => {
            let len = get_decimal(cursor)? as usize;

            let data = Bytes::copy_from_slice(&cursor.chunk()[..len]);

            skip(cursor, (len + 2) as usize)?;
            Ok(Frame::Bulk(data))
        }
        b'*' => {
            let len = get_decimal(cursor)?;

            let mut vec = vec![];

            for _ in 0..len {
                vec.push(parse(cursor)?);
            }
            Ok(Frame::Array(vec))
        }

        _ => {
            Err("error".into())
        }
    }
}

fn get_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !cursor.has_remaining() {
        return Err(FrameError::Incomplete);
    }
    Ok(cursor.get_u8())
}

fn peek_u8(cursor: &Cursor<&[u8]>) -> Result<u8, FrameError> {
    if !cursor.has_remaining() {
        return Err(FrameError::Incomplete);
    }
    Ok(cursor.chunk()[0])
}

fn get_line(cursor: &mut Cursor<&[u8]>) -> Result<String, FrameError> {
    let start = cursor.position() as usize;
    let end = cursor.get_ref().len() - 1;

    for i in start..end {
        if cursor.get_ref()[i] == b'\r' && cursor.get_ref()[i + 1] == b'\n' {
            cursor.set_position((i + 2) as u64);

            let str_vec = cursor.get_ref()[start..i].to_vec();
            let string = String::from_utf8(str_vec)?;
            return Ok(string);
        }
    }
    Err(FrameError::Incomplete)
}


fn get_decimal(cursor: &mut Cursor<&[u8]>) -> Result<i64, FrameError> {
    let line = get_line(cursor)?;

    atoi::atoi::<i64>(line.as_bytes()).ok_or_else(|| "invalid frame format".into())
}

fn skip(cursor: &mut Cursor<&[u8]>, n: usize) -> Result<(), FrameError> {
    if (cursor.remaining() < n) {
        return Err(FrameError::Incomplete);
        ;
    }
    cursor.advance(n);
    Ok(())
}

impl From<FromUtf8Error> for FrameError {
    fn from(value: FromUtf8Error) -> Self {
        "protocal error; invalid frame format".into()
    }
}

impl From<String> for FrameError {
    fn from(value: String) -> Self {
        "error".into()
    }
}

impl From<&str> for FrameError {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl Display for FrameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Incomplete => { std::fmt::Display::fmt("stream ended early", f) }
            FrameError::EndOfStream => { std::fmt::Display::fmt("attempt to extract a value failed", f) }
        }
    }
}

impl std::error::Error for FrameError {}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use bytes::{Buf, Bytes};

    use crate::frame::{check, Frame, FrameIter, get_decimal, get_line, get_u8, peek_u8};

    #[test]
    fn get_line_test() {
        let str = "abcd\r\n".as_bytes();
        let mut cursor = Cursor::new(str);

        let result = get_line(&mut cursor).unwrap();

        println!();
    }

    #[test]
    fn get_decimal_test() {
        let str = "-100\r\n".as_bytes();
        let mut cursor = Cursor::new(str);
        let result = get_decimal(&mut cursor).unwrap();
        assert_eq!(result, -100);
        println!();
    }

    #[test]
    fn get_u8_test() {
        let str = "-100\r\n".as_bytes();
        let mut cursor = Cursor::new(str);
        assert_eq!(get_u8(&mut cursor), Ok(b'-'));
        assert_eq!(get_u8(&mut cursor), Ok(b'1'));
        assert_eq!(get_u8(&mut cursor), Ok(b'0'));
        assert_eq!(get_u8(&mut cursor), Ok(b'0'));
        println!();
    }

    #[test]
    fn peek_u8_test() {
        let str = "-100\r\n".as_bytes();
        let mut cursor = Cursor::new(str);
        assert_eq!(peek_u8(&mut cursor), Ok(b'-'));
        assert_eq!(peek_u8(&mut cursor), Ok(b'-'));
        assert_eq!(peek_u8(&mut cursor), Ok(b'-'));
        assert_eq!(peek_u8(&mut cursor), Ok(b'-'));
    }


    #[test]
    fn check_test() {
        // let mut protocol1 = Cursor::new("+abcd\r\n".as_bytes());
        //
        // assert_eq!(check(&mut protocol1), Ok(()));
        //
        // let mut protocol2 = Cursor::new("-error\r\n".as_bytes());
        // assert_eq!(check(&mut protocol2), Ok(()));
        //
        // let mut protocol3 = Cursor::new(":100\r\n".as_bytes());
        // assert_eq!(check(&mut protocol3), Ok(()));
        //
        // let mut protocol4 = Cursor::new("$4\r\nabcd\r\n".as_bytes());
        // assert_eq!(check(&mut protocol4), Ok(()));

        let mut protocol5 = Cursor::new("*3\r\n+abcd\r\n-error\r\n:100\r\n".as_bytes());
        assert_eq!(check(&mut protocol5), Ok(()));

        println!();
    }


    #[test]
    fn parse_protocol_test() {
        let bytes1 = "+abcd\r\n".as_bytes();

        let frame1 = Frame::parse_protocol(bytes1).unwrap();

        let bytes2 = "-error\r\n".as_bytes();

        let frame2 = Frame::parse_protocol(bytes2).unwrap();

        let bytes3 = ":1000\r\n".as_bytes();

        let frame3 = Frame::parse_protocol(bytes3).unwrap();

        let bytes4 = "$3\r\nabc\r\n".as_bytes();

        let frame4 = Frame::parse_protocol(bytes4).unwrap();

        let bytes5 = b"*3\r\n+abcd\r\n-error\r\n:100\r\n";
        let frame5 = Frame::parse_protocol(bytes5).unwrap();


        println!("");
    }

    #[test]
    fn to_protocol_test() {
        let frame = Frame::Simple("simple".to_string());

        let vec = frame.to_protocol().unwrap();

        let result = String::from_utf8(vec.to_vec()).unwrap();
        assert_eq!(result, "+simple\r\n".to_string());

        let frame = Frame::Integer(100);

        let vec = frame.to_protocol().unwrap();


        let result = String::from_utf8(vec.to_vec()).unwrap();
        assert_eq!(result, ":100\r\n".to_string());

        let frame = Frame::Bulk(Bytes::from("test".to_string()));

        let vec = frame.to_protocol().unwrap();
        let result = String::from_utf8(vec.to_vec()).unwrap();
        println!();
    }

    #[test]
    fn frame_iter_test() {
        let vec = vec![Frame::Simple("String".to_string()), Frame::Simple("2".to_string())];

        let frame_iter = FrameIter::new(vec);

        println!();
    }
}


