use super::PluginGuest;
use super::wit::compiler;
use crate::plugin;

pub trait CompilerComponent {
    fn before_compile(&self, source: String, _ty: compiler::FileType) -> String {
        source
    }

    fn after_compile(&self, compiled: String, _ty: compiler::FileType) -> String {
        compiled
    }
}

impl compiler::Guest for PluginGuest {
    fn before_compile(source: String, ty: compiler::FileType) -> String {
        plugin().compiler_component().before_compile(source, ty)
    }

    fn after_compile(compiled: String, ty: compiler::FileType) -> String {
        plugin().compiler_component().after_compile(compiled, ty)
    }
}
