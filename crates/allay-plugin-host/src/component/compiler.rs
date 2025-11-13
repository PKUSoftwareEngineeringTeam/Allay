use super::wit::compiler::FileType;
use crate::PluginHost;
use allay_base::template::TemplateKind;

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
    pub fn before_compile(&mut self, source: String, ty: TemplateKind) -> String {
        self.plugin
            .allay_plugin_compiler()
            .call_before_compile(&mut self.store, &source, ty.into())
            .unwrap_or(source)
    }

    pub fn after_compile(&mut self, compiled: String, ty: TemplateKind) -> String {
        self.plugin
            .allay_plugin_compiler()
            .call_after_compile(&mut self.store, &compiled, ty.into())
            .unwrap_or(compiled)
    }
}
