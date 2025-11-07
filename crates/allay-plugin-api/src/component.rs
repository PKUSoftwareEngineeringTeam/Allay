use crate::base::exports::allay::allay_plugin_base::allay_plugin_base;
use crate::compiler::exports::allay::allay_plugin_compiler::allay_plugin_compiler;
use crate::compiler::exports::allay::allay_plugin_compiler::allay_plugin_compiler::FileType;
use crate::{plugin, plugin_info};

pub struct Component;

impl allay_plugin_base::Guest for Component {
    fn name() -> String {
        plugin_info().name.to_string()
    }

    fn version() -> String {
        plugin_info().version.to_string()
    }
}

impl allay_plugin_compiler::Guest for Component {
    fn before_compile(source: String, ty: FileType) -> String {
        plugin().before_compile(source, ty)
    }

    fn after_compile(compiled: String, ty: FileType) -> String {
        plugin().after_compile(compiled, ty)
    }
}
