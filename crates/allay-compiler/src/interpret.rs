pub mod interpreter;
mod scope;
mod traits;
mod var;

use crate::InterpretResult;
use crate::ast;
use crate::interpret::interpreter::{Interpretable, concat_res};
pub(crate) use interpreter::Interpreter;

/// Interpret the AST and return the rendered HTML string
pub(crate) fn interpret_template(
    ast: &ast::File,
    interpreter: &mut Interpreter,
) -> InterpretResult<String> {
    let mut res = Vec::new();
    ast.interpret(interpreter, &mut res)?;
    Ok(concat_res(&res))
}
