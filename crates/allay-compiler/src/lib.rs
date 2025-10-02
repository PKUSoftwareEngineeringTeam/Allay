#![doc = include_str!("../../../doc/dev/compiler.md")]

mod ast;
mod env;
mod error;
mod interpret;
mod misc;
mod parse;

use allay_base::config::{get_allay_config, get_theme_path};
use allay_base::data::AllayObject;
use allay_base::file;
use env::Page;
pub use error::*;
use interpret::Interpreter;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod magic {
    //! Common magic words used in Allay templates

    pub const INNER: &str = "inner";
    pub const CONTENT: &str = "content";
}

#[derive(Debug, Clone, Default)]
pub struct CompileOutput {
    /// The compiled HTML string
    pub html: String,
    /// The metadata extracted from the source file, if any
    pub meta: Option<AllayObject>,
}

/// The main Allay compiler structure with caching optimization.
/// See all the implementations in [`misc`] submodule.
#[derive(Default)]
pub struct Compiler<K: Hash + Eq> {
    /// A mapping from source files to the set of cache keys they influence.
    influenced: HashMap<PathBuf, HashSet<K>>,
    /// A mapping from cache keys to their compiled pages.
    cached: HashMap<K, Arc<Mutex<Page>>>,
}

impl<K> Compiler<K>
where
    K: Hash + Eq,
{
    /// Create a new compiler instance with default settings.
    fn default_interpreter() -> Interpreter {
        let theme = file::workspace(get_theme_path());
        let include_dir = theme.join(&get_allay_config().theme.template.dir);
        let shortcode_dir = theme.join(&get_allay_config().shortcode.dir);
        Interpreter::new(include_dir, shortcode_dir)
    }

    /// Add a listener for a source file, so that when the source file is modified,
    /// all cached pages depending on it will be cleared.
    fn add<P: AsRef<Path>>(&mut self, source: P, key: K) {
        self.influenced.entry(source.as_ref().into()).or_default().insert(key);
    }

    /// Mark a source file as modified, so that all cached pages depending on it will be cleared.
    /// This is useful when a source file is changed.
    fn modify<P: AsRef<Path>>(&mut self, source: P) {
        if let Some(deps) = self.influenced.get(source.as_ref()) {
            for dep in deps {
                let mut page = self.cached.get(dep).unwrap().lock().unwrap();
                page.clear();
            }
        }
    }

    /// Remove a source file from the cache and influenced mapping.
    /// This is useful when a source file is deleted.
    fn remove<P: AsRef<Path>>(&mut self, source: P) {
        if let Some(deps) = self.influenced.remove(source.as_ref()) {
            for dep in deps {
                self.cached.remove(&dep);
            }
        }
    }

    /// Remember a compiled page with the given key.
    fn remember(&mut self, key: K, page: Arc<Mutex<Page>>) {
        self.cached.insert(key, page);
    }
}
