#![doc = include_str!("../../../doc/dev/compiler.md")]

mod ast;
mod env;
mod error;
mod interpret;
mod meta;
mod misc;
mod parse;

use allay_base::config::{get_allay_config, get_theme_config, get_theme_path};
use allay_base::data::AllayObject;
use allay_base::file;
use env::{Compiled, Page};
pub use error::*;
use interpret::Interpreter;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

mod magic {
    //! Common magic words used in Allay templates

    /// the inner keyword for shortcode
    pub const INNER: &str = "inner";
    /// the markdown content embedded in the template
    pub const CONTENT: &str = "content";
    /// the URL of the page (auto-generated if not specified)
    pub const URL: &str = "url";
    /// the template used for an article (default `template`-`content` key defined in allay config)
    pub const TEMPLATE: &str = "template";
    /// whether to treat the content as raw Markdown, which means it will not be compiled (default `false`)
    pub const RAW: &str = "raw";
    /// if the post is hidden from listings (default `false`)
    pub const HIDDEN: &str = "hidden";
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
    /// this includes the direct publish mapping like "foo.md" -> "foo.html"
    /// together with dependencies like "page.html" -> all posts
    influenced: HashMap<PathBuf, HashSet<K>>,
    /// Pages that will be published. This is 1-to-1 mapping from source files to keys.
    /// like "foo.md" -> "foo.html"
    /// When check global refreshing, only these pages are checked.
    published: HashMap<PathBuf, K>,
    /// A mapping from keys to their compiled pages.
    cached: RefCell<HashMap<K, Arc<Mutex<Page>>>>,
}

impl<K> Compiler<K>
where
    K: Hash + Eq + Clone,
{
    /// Try to get a cached page by its key.
    fn cache(&self, key: &K) -> Option<Arc<Mutex<Page>>> {
        self.cached.borrow().get(key).cloned()
    }

    /// Remember a compiled page with the given key.
    fn remember(&mut self, key: K, page: Arc<Mutex<Page>>) {
        self.cached.borrow_mut().insert(key, page);
    }

    /// Record a publish mapping from source file to key.
    /// Also add a listener for the source file.
    fn publish(&mut self, source: impl AsRef<Path>, key: K) {
        self.listen(&source, key.clone());
        self.published.insert(source.as_ref().into(), key);
    }

    /// Add a listener for a source file, so that when the source file is modified,
    /// all cached pages depending on it will be cleared.
    fn listen(&mut self, source: impl AsRef<Path>, key: K) {
        self.influenced.entry(source.as_ref().into()).or_default().insert(key);
    }

    /// Recompile the changed pages.
    pub fn refresh_pages(&self) -> HashMap<PathBuf, CompileResult<CompileOutput>> {
        let mut results = HashMap::new();

        for (path, k) in self.published.iter() {
            if let Some(page) = self.cache(k)
                && page.lock().unwrap().changed()
            {
                let res = page.compile(&mut Self::default_interpreter());
                results.insert(path.clone(), res);
            }
        }

        results
    }

    /// Create a new compiler instance with default settings.
    fn default_interpreter() -> Interpreter {
        let theme = file::workspace(get_theme_path());
        let include_dir = theme.join(&get_theme_config().config.templates.dir);
        let shortcode_dir = theme.join(&get_allay_config().shortcode_dir);
        Interpreter::new(include_dir, shortcode_dir)
    }

    /// Mark a source file as modified, so that all cached pages depending on it will be cleared.
    /// This is useful when a source file is changed.
    pub fn modify<P: AsRef<Path>>(&mut self, source: P) {
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
    pub fn remove<P: AsRef<Path>>(&mut self, source: P) {
        if let Some(deps) = self.influenced.remove(source.as_ref()) {
            self.published.remove(source.as_ref());
            for dep in deps {
                self.cached.borrow_mut().remove(&dep);
            }
        }
    }
}
