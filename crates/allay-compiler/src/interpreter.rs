use crate::ast;
use crate::ast::Control;
use crate::error::InterpretResult;
use crate::scope::PageScope;
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
