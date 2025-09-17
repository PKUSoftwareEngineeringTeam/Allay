pub(crate) mod scope;
mod var;

use crate::InterpretResult;
use crate::ast::{self, Control};
use scope::PageScope;
use std::path::Path;

pub(crate) fn interpret_template(
    ast: &ast::File,
    _include_dir: &Path,
    _short_code_dir: &Path,
    _scope: &PageScope,
) -> InterpretResult<(String, bool)> {
    if ast.0.controls.is_empty() {
        return Ok(("".to_string(), false));
    }
    if ast.0.controls.len() == 1
        && let Control::Text(text) = ast.0.controls.first().unwrap()
    {
        return Ok((text.clone(), false));
    }
    todo!()
}
