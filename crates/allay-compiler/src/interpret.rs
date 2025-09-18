mod scope;
mod traits;
mod var;

use crate::InterpretResult;
use crate::ast::{self, Control};
use std::path::Path;

/// Interpret the AST and return the rendered HTML string
pub(crate) fn interpret_template(
    ast: &ast::File,
    _include_dir: &Path,
    _shortcode_dir: &Path,
) -> InterpretResult<String> {
    if ast.0.controls.is_empty() {
        return Ok("".into());
    }
    if ast.0.controls.len() == 1
        && let Control::Text(text) = ast.0.controls.first().unwrap()
    {
        return Ok(text.clone());
    }
    todo!()
}
