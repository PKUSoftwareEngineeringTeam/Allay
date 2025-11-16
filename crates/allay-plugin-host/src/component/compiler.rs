use super::wit::compiler::FileType;
use crate::PluginHost;
use allay_base::template::TemplateKind;
use wasmtime::AsContextMut;

impl From<TemplateKind> for FileType {
    fn from(value: TemplateKind) -> Self {
        match value {
            TemplateKind::Html => FileType::Html,
            TemplateKind::Markdown => FileType::Markdown,
            TemplateKind::Other(_) => FileType::Markdown, // Default to Markdown for unknown types
        }
    }
}

impl PluginHost {
    pub fn before_compile(&self, source: String, ty: TemplateKind) -> String {
        let mut store = self.store.blocking_lock();
        self.plugin
            .allay_plugin_compiler()
            .call_before_compile(store.as_context_mut(), &source, ty.into())
            .unwrap_or(source)
    }

    pub fn after_compile(&self, compiled: String, ty: TemplateKind) -> String {
        let mut store = self.store.blocking_lock();
        self.plugin
            .allay_plugin_compiler()
            .call_after_compile(store.as_context_mut(), &compiled, ty.into())
            .unwrap_or(compiled)
    }
}
