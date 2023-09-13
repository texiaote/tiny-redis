use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};

use bytes::Bytes;
use tokio::time;
use tokio::sync::broadcast::Receiver;
use tokio::sync::Notify;
use tokio::time::Instant;
use crate::RedisResult;

pub(crate) type SharedDb = Arc<Db>;

#[derive(Debug)]
pub(crate) struct Db {
    shared: Mutex<Store>,
    background_task: Notify,
    notify_shutdown: Receiver<()>,
}

#[derive(Debug)]
pub(crate) struct Store {
    entries: HashMap<String, Entry>,
    expirations: BTreeSet<(Instant, String)>,
}

#[derive(Debug)]
struct Entry {
    data: RedisDataType,
    expire_at: Option<Instant>,
}

#[derive(Debug)]
enum RedisDataType {
    Bytes(Bytes),
    List(Vec<Bytes>),
    Set(HashSet<Bytes>),
    SortedSet(BTreeSet<Bytes>),
    HASH(HashMap<String, Bytes>),
    BITMAP,
}

impl Store {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            expirations: BTreeSet::new(),
        }
    }

    pub(crate) fn set_bytes(&mut self, key: impl ToString, value: Bytes, expire_at: Option<Instant>) -> Option<Bytes> {
        let key = key.to_string();

        let prev_entry = self.entries.insert(key.clone(), (value, expire_at).into());

        // 如果有超时时间，则将该时间存储进来
        if let Some(expire_at) = expire_at {
            self.expirations.insert((expire_at, key.clone()));
        }

        if let Some(data) = prev_entry {
            if let Some(expire_at) = data.expire_at {
                // 删掉
                self.expirations.remove(&(expire_at, key));
            }

            if let RedisDataType::Bytes(data) = data.data {
                return Some(data);
            }
        }

        None
    }

    pub(crate) fn get_bytes(&self, key: impl ToString) -> Option<Bytes> {
        let key = key.to_string();

        if let Some(entry) = self.entries.get(&key) {
            //判断时间,时间过期了，则不能再继续了
            if let Some(expire_at) = entry.expire_at {
                if expire_at < Instant::now() {
                    return None;
                }
            }
            if let RedisDataType::Bytes(data) = &entry.data {
                return Some(Bytes::copy_from_slice(data));
            }
        }

        None
    }
    pub(crate) fn update_bytes(&mut self, key: impl ToString, value: Bytes) {
        let key = key.to_string();

        if let Some(entry) = self.entries.get_mut(&key) {
            entry.data = RedisDataType::Bytes(value);
        }
    }

    pub(crate) fn remove_vec(&mut self, keys: &Vec<String>) {
        for key in keys {
            self.remove(key);
        }
    }

    pub(crate) fn remove(&mut self, key: &str) -> Option<Bytes> {
        let prev = self.entries.remove(key);
        //去掉在expiration中对应的信息
        if let Some(prev) = prev {
            if let Some(when) = prev.expire_at {
                self.expirations.remove(&(when, key.to_string()));
            }
            if let RedisDataType::Bytes(data) = prev.data {
                return Some(data);
            }
        }
        None
    }
}

impl Db {
    pub(crate) fn new(notify_shutdown: Receiver<()>) -> SharedDb {
        let db = Arc::new(Self { shared: Mutex::new(Store::new()), background_task: Notify::new(), notify_shutdown });

        tokio::spawn(purge_expired_tasks(db.clone()));
        db
    }


    pub(crate) fn lock(&self) -> MutexGuard<Store> {
        self.shared.lock().unwrap()
    }
    // 获取key信息

    /// purge all expired keys and return the Instant at which the next
    /// key will expire
    /// The background task will sleep until this instant
    fn purge_expired_keys(&self) -> Option<Instant> {
        let mut shared = self.shared.lock().unwrap();

        let shared = &mut *shared;

        let now = Instant::now();

        while let Some(&(when, ref key)) = shared.expirations.iter().next() {
            if when > now {
                return Some(when);
            }
            //已经过期了，可以去掉了
            shared.entries.remove(key);
            shared.expirations.remove(&(when, key.clone()));
        }
        None
    }
}

impl Entry {}

impl From<(Bytes, Option<Instant>)> for Entry {
    fn from(value: (Bytes, Option<Instant>)) -> Self {
        Self {
            data: RedisDataType::Bytes(value.0),
            expire_at: value.1,
        }
    }
}

impl From<(Vec<Bytes>, Option<Instant>)> for Entry {
    fn from(value: (Vec<Bytes>, Option<Instant>)) -> Self {
        Self {
            data: RedisDataType::List(value.0),
            expire_at: value.1,
        }
    }
}


//作用于后台，异步处理过期的key数据
async fn purge_expired_tasks(db: SharedDb) {
    loop {

        //获取到过期的keys
        if let Some(when) = db.purge_expired_keys() {
            //睡眠，直到那个时间为止
            time::sleep_until(when);
        }

        //等待其他任务，把自己叫醒
        db.background_task.notified().await;

        //找到所有时间小于单签时间的任务，并将它删除掉

        //然后通知后台进行删除掉
    }
}

#[cfg(test)]
mod test {
    use tokio::sync::broadcast;

    use crate::db::{Db, SharedDb};

    fn init_db() -> SharedDb {
        let (sender, _) = broadcast::channel(1);
        Db::new(sender.subscribe())
    }
}