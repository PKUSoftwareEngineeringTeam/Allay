#![allow(dead_code)] // TODO: Remove this line when the module is fully utilized.

use std::path::Path;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TemplateKind {
    Markdown,
    Html,
    #[default]
    Other,
}

impl TemplateKind {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "md" => TemplateKind::Markdown,
            "html" | "htm" => TemplateKind::Html,
            _ => TemplateKind::Other,
        }
    }

    pub fn from_filename<P: AsRef<Path>>(filename: P) -> Self {
        let filename = filename.as_ref();
        if let Some(ext) = filename.extension().and_then(|e| e.to_str()) {
            TemplateKind::from_extension(ext)
        } else {
            TemplateKind::default()
        }
    }
}
