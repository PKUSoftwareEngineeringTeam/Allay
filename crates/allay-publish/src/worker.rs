use super::generator::{FileGenerator, FileMapper};
use allay_base::config::{get_allay_config, get_theme_config, get_theme_path};
use allay_base::template::{FileKind, TemplateKind};
use std::path::Path;
use std::{path::PathBuf, thread::spawn};

/// A worker that manages multiple file generators
#[derive(Default)]
pub struct GeneratorWorker {
    gens: Vec<Box<dyn FileGenerator>>,
}

impl GeneratorWorker {
    /// Create a new generator worker with all built-in generators
    pub fn create() -> Self {
        Self {
            gens: vec![
                Box::new(ArticleGenerator),
                Box::new(CustomGenerator),
                Box::new(StaticGenerator),
                Box::new(ThemeStaticGenerator),
                Box::new(WrapperGenerator),
            ],
        }
    }

    /// Start the generator worker
    pub fn start(&'static self) {
        for g in self.gens.iter() {
            spawn(move || {
                g.generate_all();
                g.watch();
            });
        }
    }

    /// Generate all files once
    pub fn generate_once(&self) {
        for g in self.gens.iter() {
            g.generate_all();
        }
    }
}

struct StaticGenerator;

impl FileMapper for StaticGenerator {
    fn src_root(&self) -> PathBuf {
        get_allay_config().statics_dir.clone().into()
    }
}

impl FileGenerator for StaticGenerator {
    fn content_kind(&self) -> allay_base::template::FileKind {
        FileKind::Static
    }
}

struct ThemeStaticGenerator;

impl FileMapper for ThemeStaticGenerator {
    fn src_root(&self) -> PathBuf {
        get_theme_path().join(&get_theme_config().config.static_dir)
    }
}

impl FileGenerator for ThemeStaticGenerator {
    fn content_kind(&self) -> FileKind {
        FileKind::Static
    }
}

struct ArticleGenerator;

impl FileMapper for ArticleGenerator {
    fn src_root(&self) -> PathBuf {
        get_allay_config().content_dir.clone().into()
    }

    fn path_mapping(&self, src: &Path) -> PathBuf {
        let mut res = src.to_path_buf();
        if TemplateKind::from_filename(src).is_md() {
            res.set_extension(TemplateKind::Html.extension());
        }
        res
    }
}

impl FileGenerator for ArticleGenerator {
    fn content_kind(&self) -> FileKind {
        FileKind::Article
    }
}

struct CustomGenerator;

impl FileMapper for CustomGenerator {
    fn src_root(&self) -> PathBuf {
        get_theme_path().join(&get_theme_config().config.custom_dir)
    }

    fn path_mapping(&self, src: &Path) -> PathBuf {
        let mut res = src.to_path_buf();
        if TemplateKind::from_filename(src).is_md() {
            res.set_extension(TemplateKind::Html.extension());
        }
        res
    }
}

impl FileGenerator for CustomGenerator {
    fn content_kind(&self) -> FileKind {
        FileKind::Custom
    }
}

struct WrapperGenerator;

impl FileMapper for WrapperGenerator {
    fn src_root(&self) -> PathBuf {
        get_theme_path().join(&get_theme_config().config.templates.dir)
    }
}

impl FileGenerator for WrapperGenerator {
    fn content_kind(&self) -> FileKind {
        FileKind::Wrapper
    }
}
