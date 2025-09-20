mod interpreter;
mod scope;
mod traits;
mod var;

use crate::InterpretResult;
use crate::ast;
use crate::interpret::interpreter::{Interpretable, Interpreter};
use std::path::Path;

/// Interpret the AST and return the rendered HTML string
pub(crate) fn interpret_template<P: AsRef<Path>>(
    ast: &ast::File,
    include_dir: P,
    shortcode_dir: P,
) -> InterpretResult<String> {
    let mut interpreter = Interpreter::new(
        include_dir.as_ref().to_path_buf(),
        shortcode_dir.as_ref().to_path_buf(),
    );
    let mut res = Vec::new();
    ast.interpret(&mut interpreter, &mut res)?;
    Ok(res.join(" "))
}
