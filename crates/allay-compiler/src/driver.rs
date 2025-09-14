//! Compiler driver

use crate::error::CompileError;
use crate::interpreter::interpret_template;
use crate::parser::parse_template;
use crate::scope::TemplateScope;
use std::path::Path;

/// Compile the source code once, return the compiled HTML and a boolean indicating
/// whether any changes were made.
pub(super) fn compile_once(
    source: &str,
    include_dir: &Path,
    short_code_dir: &Path,
    top_level: &TemplateScope,
) -> Result<(String, bool), CompileError> {
    let ast = parse_template(source)?;
    interpret_template(&ast, include_dir, short_code_dir, top_level)
}
