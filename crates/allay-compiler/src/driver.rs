//! Compiler driver

use crate::CompileResult;
use crate::interpret::interpret_template;
use crate::parse::parse_template;
use std::path::Path;

/// Compile the source code once, return the compiled HTML
pub(super) fn compile_once<P: AsRef<Path>>(
    source: &str,
    include_dir: P,
    shortcode_dir: P,
) -> CompileResult<String> {
    let ast = parse_template(source)?;
    let res = interpret_template(&ast, include_dir, shortcode_dir)?;
    Ok(res)
}
