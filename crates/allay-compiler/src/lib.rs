#![doc = include_str!("../../../doc/dev/compiler.md")]

mod ast;
mod compile;
mod error;
mod interpret;
mod parse;

use crate::compile::Compiled;
use compile::Page;
pub use error::*;
use interpret::Interpreter;
use std::{cell::RefCell, path::Path, rc::Rc};

/// Compile a source file (markdown or html) into HTML string.
///
/// # Arguments
/// - `source`: The path to the source file (markdown or html)
/// - `include_dir`: The directory to look for included templates
/// - `shortcode_dir`: The directory to look for shortcodes
///
/// # Returns
/// The compiled HTML string or a [`CompileError`]
pub fn compile<P: AsRef<Path>>(
    source: P,
    include_dir: P,
    shortcode_dir: P,
) -> CompileResult<String> {
    let mut interpreter = Interpreter::new(
        include_dir.as_ref().to_path_buf(),
        shortcode_dir.as_ref().to_path_buf(),
    );
    let page = Page::new(source.as_ref().to_path_buf());
    let page = Rc::new(RefCell::new(page));
    page.compile(&mut interpreter)
}
