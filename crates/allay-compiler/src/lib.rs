#![doc = include_str!("../../../doc/dev/compiler.md")]

mod ast;
mod env;
mod error;
mod interpret;
mod meta;
mod misc;
mod parse;

use allay_base::config::{get_allay_config, get_theme_path};
use allay_base::data::AllayObject;
use allay_base::file;
use env::{Compiled, Page};
pub use error::*;
use interpret::Interpreter;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod magic {
    //! Common magic words used in Allay templates

    pub const INNER: &str = "inner";
    pub const CONTENT: &str = "content";
    pub const URL: &str = "url";
}

#[derive(Debug, Clone, Default)]
pub struct CompileOutput {
    /// The compiled HTML string
    pub html: String,
    /// The metadata extracted from the source file
    pub meta: AllayObject,
}

/// The main Allay compiler structure with caching optimization.
/// See all the implementations in `misc` submodule.
#[derive(Default)]
pub struct Compiler<K> {
    /// A mapping from source files to the set of keys they influence.
    influenced: HashMap<PathBuf, HashSet<K>>,
    /// A mapping from keys to their compiled pages.
    cached: RefCell<HashMap<K, Arc<Mutex<Page>>>>,
    /// Pages that will be published
    published: HashSet<K>,
}

impl<K> Compiler<K>
where
    K: Hash + Eq,
{
    /// Try to get a cached page by its key.
    fn cache(&self, key: &K) -> Option<Arc<Mutex<Page>>> {
        self.cached.borrow().get(key).cloned()
    }

    /// Remember a compiled page with the given key.
    fn remember(&mut self, key: K, page: Arc<Mutex<Page>>) {
        self.cached.borrow_mut().insert(key, page);
    }

    /// Recompile the changed pages.
    pub fn refresh_pages<P: AsRef<Path>>(
        &self,
        skip: P,
    ) -> HashMap<PathBuf, CompileResult<CompileOutput>> {
        let mut results = HashMap::new();
        for k in self.published.iter() {
            let page = self.cache(k);
            if let Some(page) = page
                && page.lock().unwrap().changed()
            {
                for (buf, set) in self.influenced.iter() {
                    if buf != &skip.as_ref().to_path_buf() && set.contains(k) {
                        let res = page.compile(&mut Self::default_interpreter());
                        results.insert(buf.clone(), res);
                        break;
                    }
                }
            }
        }
        results
    }

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
                if let Some(page) = self.cache(dep) {
                    let mut page = page.lock().unwrap();
                    page.clear();
                }
            }
        }
    }

    /// Remove a source file from the cache and influenced mapping.
    /// This is useful when a source file is deleted.
    fn remove<P: AsRef<Path>>(&mut self, source: P) {
        if let Some(deps) = self.influenced.remove(source.as_ref()) {
            for dep in deps {
                self.published.remove(&dep);
                self.cached.borrow_mut().remove(&dep);
            }
        }
    }
}
