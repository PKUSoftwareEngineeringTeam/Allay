#![doc = include_str!("../../../doc/dev/compiler.md")]

mod ast;
mod error;
mod interpret;
mod parse;

use allay_base::{file, template::TemplateKind};
pub use error::*;
use interpret::{Interpreter, interpret_template};
use parse::parse_template;
use pulldown_cmark::{Parser, html};
use std::path::Path;

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
    compile_on(source, &mut interpreter)
}

fn compile_on<P: AsRef<Path>>(source: P, interpreter: &mut Interpreter) -> CompileResult<String> {
    let source_path = source.as_ref();
    let kind = TemplateKind::from_filename(source_path);
    let content = match kind {
        TemplateKind::Html | TemplateKind::Markdown => file::read_file_string(source_path)?,
        TemplateKind::Other(e) => return Err(CompileError::FileTypeNotSupported(e)),
    };
    let ast = parse_template(&content)?;
    let res = interpret_template(&ast, interpreter)?;
    let res = match kind {
        TemplateKind::Markdown => convert_to_html(&res)?,
        _ => res,
    };
    Ok(res)
}

fn convert_to_html(text: &str) -> CompileResult<String> {
    let mut html_output = String::new();
    html::push_html(&mut html_output, Parser::new(text));
    Ok(html_output)
}
