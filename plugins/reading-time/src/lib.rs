use allay_plugin_api::{ListenComponent, Plugin, register_plugin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{RwLock, RwLockWriteGuard};

#[derive(Serialize, Deserialize, Default)]
struct ReadingTime {
    entries: HashMap<PathBuf, u32>,
}

struct ReadingTimeGenerator {
    data: RwLock<ReadingTime>,
}

impl ReadingTimeGenerator {
    const INPUT: &'static str = "contents";
    const OUTPUT: &'static str = "public";
    const FILENAME: &'static str = "reading_time.json";

    fn new() -> Self {
        Self {
            data: RwLock::new(ReadingTime::default()),
        }
    }

    fn file(&self) -> PathBuf {
        PathBuf::from(Self::OUTPUT).join(Self::FILENAME)
    }

    fn path_of(source: impl AsRef<Path>) -> PathBuf {
        PathBuf::from(Self::INPUT).join(source)
    }

    fn estimate_reading_time(&self, content: &str) -> u32 {
        let word_count = content.chars().filter(|c| c.is_alphanumeric()).count();
        let chinese_count = content
            .chars()
            .filter(|c| (*c as u32) >= 0x4E00 && (*c as u32) <= 0x9FFF)
            .count();

        let english_words = word_count - chinese_count;
        let chinese_minutes = (chinese_count as f64 / 500.0).ceil() as u32;
        let english_minutes = (english_words as f64 / 300.0).ceil() as u32;

        chinese_minutes + english_minutes
    }

    fn write(&self) -> RwLockWriteGuard<'_, ReadingTime> {
        self.data.write().expect("lock poisoned")
    }

    fn dump(&self) {
        let json = {
            let data = self.data.read().expect("lock poisoned");
            serde_json::to_string_pretty(&*data).unwrap()
        };
        fs::write(self.file(), json).expect("Unable to write reading time file");
    }
}

impl ListenComponent for ReadingTimeGenerator {
    fn on_create(&self, source: String) {
        let path = Self::path_of(&source);
        if path.extension().is_none_or(|s| s != "md") {
            return;
        }

        let content = fs::read_to_string(Self::path_of(&source)).expect("Unable to read file");
        let minutes = self.estimate_reading_time(&content);
        self.write().entries.insert(source.into(), minutes);
        self.dump();
    }

    fn on_modify(&self, source: String) {
        let path = Self::path_of(&source);
        if path.extension().is_none_or(|s| s != "md") {
            return;
        }

        let content = fs::read_to_string(Self::path_of(&source)).expect("Unable to read file");
        let minutes = self.estimate_reading_time(&content);
        self.write().entries.insert(source.into(), minutes);
        self.dump();
    }

    fn on_remove(&self, source: String) {
        self.write().entries.remove(&PathBuf::from(&source));
    }
}

struct ReadingTimePlugin {
    generator: ReadingTimeGenerator,
}

impl Plugin for ReadingTimePlugin {
    fn name() -> &'static str {
        "reading-time"
    }

    fn version() -> &'static str {
        "0.1.0"
    }

    fn new() -> Self {
        Self {
            generator: ReadingTimeGenerator::new(),
        }
    }

    fn listen_component(&self) -> &dyn ListenComponent {
        &self.generator
    }
}

register_plugin!(ReadingTimePlugin);
