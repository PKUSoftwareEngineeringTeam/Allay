use crate::interpret::{Interpretable, Interpreter, PageScope};
use crate::parse::parse_template;
use crate::{CompileError, CompileResult};
use allay_base::{file, template::TemplateKind};
use pulldown_cmark::{Parser, html};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::{Rc, Weak};

/// The page environment to record the state of a page during compiling
/// Fully optimized for increment compiling
#[derive(Debug, Clone)]
pub(crate) struct Page {
    /// the parent page, if any
    parent: Option<Weak<RefCell<Page>>>,
    /// the path of the page
    path: PathBuf,
    /// the interpret scope of the page
    scope: PageScope,
    /// the output tokens
    output: Vec<Token>,

    /// if the page itself is compiled
    ready: bool,
    /// the cache of compiling
    cache: String,
    /// if the page's dependencies need recompiling
    dirty: bool,
}

#[derive(Debug, Clone)]
enum Token {
    Text(String),
    Page(Rc<RefCell<Page>>),
}

impl Page {
    /// Create a new page at the top level (root node)
    pub fn new(path: PathBuf) -> Self {
        Page {
            parent: None,
            path,
            scope: PageScope::new(),
            output: Vec::new(),

            ready: false,
            cache: String::new(),
            dirty: true,
        }
    }

    /// Create a subpage with the given parent and scope.
    /// Usually called by `include` or `shortcode`.
    pub fn create_subpage(
        parent: &Rc<RefCell<Page>>,
        path: PathBuf,
        scope: PageScope,
    ) -> Rc<RefCell<Page>> {
        let page = Page::new(path);
        let page = Page {
            parent: Some(Rc::downgrade(parent)),
            scope,
            ..page
        };
        let page = Rc::new(RefCell::new(page));
        parent.borrow_mut().output.push(Token::Page(page.clone()));
        page
    }

    pub fn scope(&self) -> &PageScope {
        &self.scope
    }

    pub fn scope_mut(&mut self) -> &mut PageScope {
        &mut self.scope
    }

    /// Detach the page from its parent,
    /// which means the parent won't be notified when this page is modified.
    pub fn detach(&mut self) {
        self.parent = None;
    }

    /// Insert text to the output
    pub fn insert(&mut self, text: String) {
        self.output.push(Token::Text(text));
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
            parent.borrow_mut().spread_dirty();
        }
    }
}

fn convert_to_html(text: &str) -> CompileResult<String> {
    let mut html_output = String::new();
    html::push_html(&mut html_output, Parser::new(text));
    Ok(html_output)
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

impl Compiled for Rc<RefCell<Page>> {
    // The optimized version for compiling a page (by caching the result)
    fn compile(&self, interpreter: &mut Interpreter) -> CompileResult<String> {
        if !self.borrow().ready {
            // compile only when modified
            let kind = TemplateKind::from_filename(&self.borrow().path);
            let content = match kind {
                TemplateKind::Html | TemplateKind::Markdown => {
                    file::read_file_string(&self.borrow().path)?
                }
                TemplateKind::Other(e) => return Err(CompileError::FileTypeNotSupported(e)),
            };
            let ast = parse_template(&content)?;
            self.compile_on(&ast, interpreter)?;
            self.borrow_mut().ready = true;
        }
        if !self.borrow().dirty {
            // use cached result
            return Ok(self.borrow().cache.clone());
        }
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
        for token in &self.borrow().output {
            result.push(' ');
            match token {
                Token::Text(t) => result.push_str(t),
                Token::Page(p) => result.push_str(&p.compile(interpreter)?),
            }
        }
        let kind = TemplateKind::from_filename(&self.borrow().path);
        if matches!(kind, TemplateKind::Markdown) {
            result = convert_to_html(&result)?;
        }

        self.borrow_mut().dirty = false;
        self.borrow_mut().cache = result.clone();
        Ok(result)
    }
}
