#![allow(dead_code)] // TODO: Remove this line when the module is fully utilized.

use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateKind {
    Markdown,
    Html,
    Other(String),
}

impl Default for TemplateKind {
    fn default() -> Self {
        TemplateKind::Other(String::new())
    }
}

impl TemplateKind {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "md" => TemplateKind::Markdown,
            "html" | "htm" => TemplateKind::Html,
            e => TemplateKind::Other(e.to_string()),
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

    pub fn extension(&self) -> &str {
        match self {
            TemplateKind::Markdown => "md",
            TemplateKind::Html => "html",
            TemplateKind::Other(e) => e.as_str(),
        }
    }
}
