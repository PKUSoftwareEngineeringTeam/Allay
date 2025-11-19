use crate::interpret::{Interpretable, Interpreter, PageScope};
use crate::meta::get_meta_and_content;
use crate::{CompileOutput, CompileResult};
use allay_base::template::TemplateKind;
#[cfg(feature = "plugin")]
use allay_plugin::PluginManager;
use pulldown_cmark::{Parser, html};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};

macro_rules! get_lock {
    ($e:expr) => {
        $e.lock().unwrap_or_else(|_| panic!("Lock poisoned! This is a bug of Allay, please report it to the developers on https://github.com/PKUSoftwareEngineeringTeam/Allay/issues with the stack trace."))
    };
}

#[derive(Debug)]
/// The page environment to record the state of a page during compiling
/// Fully optimized for increment compiling
pub struct Page {
    /// the parent page, if any
    parent: Option<Weak<Mutex<Page>>>,
    /// the path of the page
    path: PathBuf,
    /// the interpret scope of the page
    scope: PageScope,
    /// subpages which are planned to be added to output when compiling
    /// like the ".inner", ".content" magic words in the template
    stash: HashMap<String, Arc<Mutex<Page>>>,
    /// the output tokens
    output: Vec<Token>,

    /// if the page is cachable
    cachable: bool,
    /// if the page itself is compiled
    ready: bool,
    /// the cache of compiling
    cache: CompileOutput,
    /// if the page's dependencies need recompiling
    dirty: bool,
}

#[derive(Debug)]
enum Token {
    Text(String),
    Page(Arc<Mutex<Page>>),
}

impl Page {
    /// Create a new page at the top level (root node)
    pub fn new(path: PathBuf) -> Self {
        Page {
            parent: None,
            path,
            scope: PageScope::new(),
            stash: HashMap::new(),
            output: Vec::new(),

            cachable: true,
            ready: false,
            cache: CompileOutput::default(),
            dirty: true,
        }
    }

    pub fn scope(&self) -> &PageScope {
        &self.scope
    }

    pub fn scope_mut(&mut self) -> &mut PageScope {
        &mut self.scope
    }

    pub fn set_cachable(&mut self, cachable: bool) {
        self.cachable = cachable;
        self.ready = false;
        self.dirty = true;
        if !cachable
            && let Some(parent) = &self.parent
            && let Some(parent) = parent.upgrade()
        {
            let mut parent = get_lock!(parent);
            if parent.cachable {
                parent.set_cachable(false);
            }
        }
    }

    /// Check if the page's output is changed and needs recompiling
    pub fn changed(&self) -> bool {
        !self.cachable || self.dirty
    }

    /// Clone the page without parent and output
    pub fn clone_detached(&self) -> Self {
        Page {
            parent: None,
            path: self.path.clone(),
            scope: self.scope.clone(),
            stash: self.stash.clone(),
            output: Vec::new(),

            cachable: self.cachable,
            ready: false,
            cache: CompileOutput::default(),
            dirty: true,
        }
    }

    /// Stash a subpage with the given key.
    pub fn add_stash(&mut self, key: String, page: Arc<Mutex<Page>>) {
        self.stash.insert(key, page);
    }

    /// Clear the compiled state, so that the page will be recompiled on next `compile` call.
    /// This will also mark the parent page as dirty, so that it and its parents will be recompiled too.
    pub fn clear(&mut self) {
        self.ready = false;
        self.spread_dirty();
    }

    /// Spread the dirty state to parent pages
    fn spread_dirty(&mut self) {
        self.dirty = true;
        self.output.clear();
        if let Some(parent) = &self.parent
            && let Some(parent) = parent.upgrade()
        {
            let mut parent = get_lock!(parent);
            if !parent.dirty {
                parent.spread_dirty();
            }
        }
    }

    /// Convert the page into a reference-counted pointer
    pub fn into(self) -> Arc<Mutex<Page>> {
        Arc::new(Mutex::new(self))
    }
}

fn convert_to_html(text: &str) -> CompileResult<String> {
    let mut html_output = String::new();
    html::push_html(&mut html_output, Parser::new(text));
    Ok(html_output)
}

pub(crate) trait TokenInserter: Sized {
    /// Insert text to the output
    fn insert_text(&self, text: String);

    /// Insert a subpage with the given parent and scope.
    /// Usually called by `include` or `shortcode`.
    /// This page's reference count will be returned.
    fn insert_subpage(&self, path: PathBuf, scope: PageScope) -> Self;

    /// Insert a stashed page with the given key.
    /// This page's reference count will be returned if the key exists.
    fn insert_stash(&self, key: &str) -> Option<Self>;
}

impl TokenInserter for Arc<Mutex<Page>> {
    fn insert_text(&self, text: String) {
        get_lock!(self).output.push(Token::Text(text));
    }

    fn insert_subpage(&self, path: PathBuf, scope: PageScope) -> Self {
        let page = Page::new(path);
        let page = Page {
            parent: Some(Arc::downgrade(self)),
            scope,
            ..page
        };
        let page = page.into();
        get_lock!(self).output.push(Token::Page(page.clone()));
        page
    }

    fn insert_stash(&self, key: &str) -> Option<Self> {
        let page = { get_lock!(self).stash.get(key)?.clone() };
        get_lock!(page).parent = Some(Arc::downgrade(self));
        get_lock!(self).output.push(Token::Page(page.clone()));
        Some(page)
    }
}

pub trait Compiled {
    /// Compile the page and return the rendered HTML string
    fn compile(&self, interpreter: &mut Interpreter) -> CompileResult<CompileOutput>;
    /// Compile the page on the given AST node in the page
    fn compile_on<T>(
        &self,
        node: &dyn Interpretable<Output = T>,
        interpreter: &mut Interpreter,
    ) -> CompileResult<String>;
    /// Utility function to generate the result string after the compiling
    fn gen_result_str(&self, interpreter: &mut Interpreter) -> CompileResult<String>;
}

#[cfg(feature = "plugin")]
fn after_compile(html: String, ty: TemplateKind) -> String {
    let plugin_manager = PluginManager::instance();
    plugin_manager.plugins().iter().fold(html, |html, plugin| {
        let mut plugin = plugin.lock().expect("Plugin lock poisoned!");
        plugin.after_compile(html, ty.clone())
    })
}

impl Compiled for Arc<Mutex<Page>> {
    // The optimized version for compiling a page (by caching the result)
    fn compile(&self, interpreter: &mut Interpreter) -> CompileResult<CompileOutput> {
        let mut page = get_lock!(self);
        let meta = if !page.cachable || !page.ready {
            let (meta, template) = get_meta_and_content(&page.path)?;
            page.scope.merge_data(meta.clone());
            page.output.clear();
            drop(page);
            self.compile_on(&template, interpreter)?;
            get_lock!(self).ready = true;
            meta
        } else {
            let meta = page.cache.meta.clone();
            drop(page);
            meta
        };

        let page = get_lock!(self);
        if !page.dirty {
            // use cached result
            return Ok(page.cache.clone());
        }
        drop(page);

        let html = self.gen_result_str(interpreter)?;

        #[cfg(feature = "plugin")]
        let html = after_compile(html, TemplateKind::from_filename(&get_lock!(self).path));

        let output = CompileOutput { html, meta };
        let mut page = get_lock!(self);
        page.dirty = false;
        page.cache = output.clone();
        Ok(output)
    }

    fn compile_on<T>(
        &self,
        node: &dyn Interpretable<Output = T>,
        interpreter: &mut Interpreter,
    ) -> CompileResult<String> {
        node.interpret(interpreter, self)?;
        self.gen_result_str(interpreter)
    }

    fn gen_result_str(&self, interpreter: &mut Interpreter) -> CompileResult<String> {
        let mut result = String::new();
        let page = get_lock!(self);
        for token in page.output.iter() {
            result.push(' ');
            match token {
                Token::Text(t) => result.push_str(t),
                Token::Page(p) => result.push_str(&p.compile(interpreter)?.html),
            }
        }
        drop(page);

        let kind = { TemplateKind::from_filename(&get_lock!(self).path) };
        if kind.is_md() {
            result = convert_to_html(&result)?;
        }

        Ok(result)
    }
}
