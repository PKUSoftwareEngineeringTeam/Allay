mod interpreter;
mod scope;
mod traits;
mod var;

use crate::{InterpretResult, ast::Meta};
use allay_base::data::{AllayData, AllayObject};
pub use interpreter::{Interpretable, Interpreter};
pub use scope::PageScope;

pub fn interpret_meta(meta: &Meta) -> InterpretResult<AllayObject> {
    let meta = match meta {
        Meta::Yaml(yaml) => AllayData::from_yaml(yaml)?,
        Meta::Toml(toml) => AllayData::from_toml(toml)?,
    };
    Ok(meta)
}
