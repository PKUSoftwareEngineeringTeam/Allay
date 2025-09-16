#![doc = include_str!("../../../doc/dev/compiler.md")]

mod ast;
mod driver;
pub mod error;
mod interpreter;
mod parser;
pub mod scope;

use crate::error::{CompileError, CompileResult};
use crate::scope::PageScope;
use allay_base::data::AllayObject;
use allay_base::file;
use pulldown_cmark::{Parser, html};
use std::path::Path;

/// Compile a source file (markdown or html) into HTML string.
///
/// # Arguments
/// - `source`: The path to the source file (markdown or html)
/// - `include_dir`: The directory to look for included templates
/// - `short_code_dir`: The directory to look for shortcodes
/// - `top_level`: The top level scope for template variables
///
/// # Returns
/// The compiled HTML string or a [`CompileError`]
pub fn compile<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    source: P,
    include_dir: Q,
    short_code_dir: R,
) -> CompileResult<String> {
    // read source file, convert to html if source is markdown
    let source_path = source.as_ref();
    let source_content = file::read_file_string(source_path)?;
    let ext = source_path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let mut source_content = if ext == "md" {
        let mut html_output = String::new();
        html::push_html(&mut html_output, Parser::new(&source_content));
        html_output
    } else if ext == "html" {
        source_content
    } else {
        return Err(CompileError::FileTypeNotSupported(ext.to_string()));
    };

    // repeatedly compile until no changes
    // TODO: add data from front-matter etc.
    let scope = PageScope::new_top(AllayObject::new());

    loop {
        let (new_content, changed) = driver::compile_once(
            &source_content,
            include_dir.as_ref(),
            short_code_dir.as_ref(),
            &scope,
        )?;
        if !changed {
            return Ok(new_content);
        }
        source_content = new_content;
    }
}
