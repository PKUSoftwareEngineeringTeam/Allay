use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub struct FileCacher<T> {
    cache: HashMap<PathBuf, (u64, T)>,
}

impl<T> FileCacher<T> {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get<P: AsRef<Path>>(&self, path: P, timestamp: u64) -> Option<&T> {
        let path = path.as_ref().to_path_buf();
        let (t, value) = self.cache.get(&path)?;
        (*t == timestamp).then_some(value)
    }

    pub fn insert<P: AsRef<Path>>(&mut self, path: P, timestamp: u64, value: T) -> Option<T> {
        let path = path.as_ref().to_path_buf();
        self.cache.insert(path, (timestamp, value)).map(|(_, v)| v)
    }
}
