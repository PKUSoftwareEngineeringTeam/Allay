use diesel::{Connection, SqliteConnection};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct ConnPool {
    pool: Vec<Mutex<SqliteConnection>>,
    idx: AtomicUsize,
}

impl ConnPool {
    pub fn new(db_url: &str, size: usize) -> Self {
        let pool = (0..size)
            .map(|_| Mutex::new(SqliteConnection::establish(db_url).unwrap()))
            .collect();
        ConnPool {
            pool,
            idx: AtomicUsize::new(0),
        }
    }

    pub fn get(&self) -> &Mutex<SqliteConnection> {
        let idx = self.idx.fetch_add(1, Ordering::SeqCst) % self.pool.len();
        &self.pool[idx]
    }
}
