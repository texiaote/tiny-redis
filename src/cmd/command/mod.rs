use crate::frame::Frame;

mod command;


pub(crate) struct CommandInfo {
    name: String,
    arith: i32,
    flags: Vec<String>,
    first_key: Option<String>,
    last_key: Option<String>,
    step: Option<i32>,
}

impl CommandInfo {
    pub(crate) fn new(name: String, arith: i32, flags: Vec<String>) -> Self {
        Self {
            name,
            arith,
            flags,
            first_key: None,
            last_key: None,
            step: None,
        }
    }

    pub(crate) fn to_frame(&self) -> Frame {
        let mut vec = vec![];
        vec.push(Frame::Simple(self.name.clone()));

        Frame::Array(vec)
    }
}