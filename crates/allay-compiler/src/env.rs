use crate::interpret::{Interpretable, Interpreter, PageScope};
use crate::parse::parse_template;
use crate::{CompileError, CompileResult};
use allay_base::{file, template::TemplateKind};
use pulldown_cmark::{Parser, html};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};

macro_rules! get_lock {
    ($e:expr) => {
        $e.lock().unwrap_or_else(|_| panic!("Lock poisoned! This is a bug of Allay, please report it to the developers on https://github.com/PKUSoftwareEngineeringTeam/Allay/issues with the stack trace."))
    };
}

/// The page environment to record the state of a page during compiling
/// Fully optimized for increment compiling
pub(crate) struct Page {
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

    /// if the page itself is compiled
    ready: bool,
    /// the cache of compiling
    cache: String,
    /// if the page's dependencies need recompiling
    dirty: bool,
}

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

            ready: false,
            cache: String::new(),
            dirty: true,
        }
    }

    pub fn scope(&self) -> &PageScope {
        &self.scope
    }

    pub fn scope_mut(&mut self) -> &mut PageScope {
        &mut self.scope
    }

    /// Clone the page without parent and output
    pub fn clone_detached(&self) -> Self {
        Page {
            parent: None,
            path: self.path.clone(),
            scope: self.scope.clone(),
            stash: self.stash.clone(),
            output: Vec::new(),
            ready: false,
            cache: String::new(),
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
            get_lock!(parent).spread_dirty();
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
        let self_page = get_lock!(self);
        let p = self_page.stash.get(key)?.clone();
        drop(self_page);

        let mut p_page = get_lock!(p);
        p_page.parent = Some(Arc::downgrade(self));
        drop(p_page);

        get_lock!(self).output.push(Token::Page(p.clone()));
        Some(p)
    }
}

pub(crate) trait Compiled {
    /// Compile the page and return the rendered HTML string
    fn compile(&self, interpreter: &mut Interpreter) -> CompileResult<String>;
    /// Compile the page on the given AST node in the page
    fn compile_on<T>(
        &self,
        node: &dyn Interpretable<Output = T>,
        interpreter: &mut Interpreter,
    ) -> CompileResult<String>;
    /// Utility function to generate the result string after the compiling
    fn gen_result_str(&self, interpreter: &mut Interpreter) -> CompileResult<String>;
}

impl Compiled for Arc<Mutex<Page>> {
    // The optimized version for compiling a page (by caching the result)
    fn compile(&self, interpreter: &mut Interpreter) -> CompileResult<String> {
        let page = get_lock!(self);
        if !page.ready {
            // compile only when modified
            let kind = TemplateKind::from_filename(&page.path);
            let content = match kind {
                TemplateKind::Html | TemplateKind::Markdown => file::read_file_string(&page.path)?,
                TemplateKind::Other(e) => return Err(CompileError::FileTypeNotSupported(e)),
            };
            drop(page);

            let ast = parse_template(&content)?;
            self.compile_on(&ast, interpreter)?;

            get_lock!(self).ready = true;
        } else {
            drop(page);
        }

        let page = get_lock!(self);
        if !page.dirty {
            // use cached result
            return Ok(page.cache.clone());
        }
        drop(page);

        self.gen_result_str(interpreter)
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
                Token::Page(p) => result.push_str(&p.compile(interpreter)?),
            }
        }
        drop(page);

        let kind = {
            let page = get_lock!(self);
            TemplateKind::from_filename(&page.path)
        };
        if matches!(kind, TemplateKind::Markdown) {
            result = convert_to_html(&result)?;
        }

        let mut page = get_lock!(self);
        page.dirty = false;
        page.cache = result.clone();
        Ok(result)
    }
}
