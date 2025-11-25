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
    pub fn is_md(&self) -> bool {
        matches!(self, TemplateKind::Markdown)
    }

    pub fn is_html(&self) -> bool {
        matches!(self, TemplateKind::Html)
    }

    pub fn is_template(&self) -> bool {
        self.is_md() || self.is_html()
    }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileKind {
    /// The article content, usually in markdown format
    Article,
    /// The article wrapper template, like `page.html`
    Wrapper,
    /// The general page template, like `index.html`
    Custom,
    /// A static file, like CSS, JS, images, etc.
    Static,
}
