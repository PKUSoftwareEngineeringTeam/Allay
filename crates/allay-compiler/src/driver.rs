//! Compiler driver

use crate::CompileResult;
use crate::interpret::interpret_template;
use crate::parse::parse_template;
use std::path::Path;

/// Compile the source code once, return the compiled HTML and a boolean indicating
/// whether any changes were made.
pub(super) fn compile_once(
    source: &str,
    include_dir: &Path,
    shortcode_dir: &Path,
) -> CompileResult<(String, bool)> {
    let ast = parse_template(source)?;
    let (res, changed) = interpret_template(&ast, include_dir, shortcode_dir)?;
    Ok((res, changed))
}
