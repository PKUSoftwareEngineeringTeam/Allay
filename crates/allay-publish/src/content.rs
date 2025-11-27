use crate::generator::{FileGenerator, FileGeneratorOptions, FileListener};
use allay_base::config::{get_allay_config, get_theme_config, get_theme_path};
use allay_base::template::FileKind;
use std::path::PathBuf;
use std::thread;

/// A worker that manages multiple file generators
#[derive(Default)]
pub struct ContentGeneratorWorker {
    gens: Vec<FileGenerator>,
}

impl ContentGeneratorWorker {
    /// Create a new generator worker with all built-in generators
    pub fn create() -> Self {
        let publish = PathBuf::from(get_allay_config().publish_dir.clone());
        let content_dir = PathBuf::from(get_allay_config().content_dir.clone());

        let article_generator = FileGeneratorOptions::default()
            .src_root(content_dir.clone())
            .dest_root(publish.clone())
            .kind(FileKind::Article)
            .to_html(true)
            .build();
        let custom_generator = FileGeneratorOptions::default()
            .src_root(get_theme_path().join(&get_theme_config().config.custom_dir))
            .dest_root(publish.clone())
            .kind(FileKind::Custom)
            .to_html(true)
            .build();
        let static_generator = FileGeneratorOptions::default()
            .src_root(get_allay_config().statics_dir.clone().into())
            .dest_root(publish.clone())
            .build();
        let theme_static_generator = FileGeneratorOptions::default()
            .src_root(get_theme_path().join(&get_theme_config().config.static_dir))
            .dest_root(publish)
            .build();
        let wrapper_generator = FileGeneratorOptions::default()
            .src_root(get_theme_path().join(&get_theme_config().config.templates.dir))
            .kind(FileKind::Wrapper)
            .build();

        Self {
            gens: vec![
                article_generator,
                custom_generator,
                static_generator,
                theme_static_generator,
                wrapper_generator,
            ],
        }
    }

    /// Start the generator worker
    pub fn start(&'static self) {
        for g in self.gens.iter() {
            thread::spawn(move || {
                g.cold_start();
                g.watch();
            });
        }
    }

    /// Generate all files once
    pub fn generate_once(&self) {
        for g in self.gens.iter() {
            g.cold_start();
        }
    }
}
