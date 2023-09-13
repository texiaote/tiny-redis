use std::error::Error;

pub mod server;
mod frame;
mod connection;
mod cmd;
mod db;


pub type RedisResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
