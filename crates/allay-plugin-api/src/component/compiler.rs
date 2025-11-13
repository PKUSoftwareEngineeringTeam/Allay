use super::PluginGuest;
use super::wit::compiler;
use crate::plugin;
use allay_base::template::TemplateKind;

impl From<compiler::FileType> for TemplateKind {
    fn from(value: compiler::FileType) -> Self {
        match value {
            compiler::FileType::Html => TemplateKind::Html,
            compiler::FileType::Markdown => TemplateKind::Markdown,
        }
    }
}

pub trait CompilerComponent {
    // TODO: use Option<String> for better performance if the plugin doesn't need to modify the source
    fn before_compile(&self, source: String, _ty: TemplateKind) -> String {
        source
    }

    fn after_compile(&self, compiled: String, _ty: TemplateKind) -> String {
        compiled
    }
}

impl compiler::Guest for PluginGuest {
    fn before_compile(source: String, ty: compiler::FileType) -> String {
        plugin().compiler_component().before_compile(source, ty.into())
    }

    fn after_compile(compiled: String, ty: compiler::FileType) -> String {
        plugin().compiler_component().after_compile(compiled, ty.into())
    }
}
