mod interpreter;
mod scope;
mod traits;
mod var;

use crate::{InterpretResult, ast::Meta};
use allay_base::data::{AllayData, AllayObject};
pub use interpreter::{Interpretable, Interpreter};
pub use scope::PageScope;
use std::sync::Arc;

/// Interpret the front matter section into an [`AllayObject`].
pub fn interpret_meta(meta: &Option<Meta>) -> InterpretResult<Arc<AllayObject>> {
    let meta = match meta {
        None => Arc::new(AllayObject::default()),
        Some(Meta::Yaml(yaml)) => AllayData::from_yaml(yaml)?,
        Some(Meta::Toml(toml)) => AllayData::from_toml(toml)?,
    };
    Ok(meta)
}
