use crate::{component::PluginGuest, plugin};
pub use compiler::FileType;
use exports::allay::plugin::compiler;

wit_bindgen::generate!({
   path: "wit/compiler.wit"
});

pub trait CompilerComponent {
    // TODO: use Option<String> for better performance if the plugin doesn't need to modify the source
    fn before_compile(&self, source: String, _ty: FileType) -> String {
        source
    }

    fn after_compile(&self, compiled: String, _ty: FileType) -> String {
        compiled
    }
}

impl compiler::Guest for PluginGuest {
    fn before_compile(source: String, ty: FileType) -> String {
        plugin().compiler_component().before_compile(source, ty)
    }

    fn after_compile(compiled: String, ty: FileType) -> String {
        plugin().compiler_component().after_compile(compiled, ty)
    }
}

export!(PluginGuest);
