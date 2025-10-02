use super::traits::{FileGenerator, FileMapper};
use allay_base::config::{get_allay_config, get_theme_path};
use allay_base::file::{self, FileResult};
use allay_base::template::{ContentKind, TemplateKind};
use allay_compiler::Compiler;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::thread::spawn;
use tracing::warn;

static COMPILER: LazyLock<Mutex<Compiler<String>>> =
    LazyLock::new(|| Mutex::new(Compiler::default()));

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
                Box::new(GeneralGenerator),
                Box::new(StaticGenerator),
            ],
        }
    }

    /// Start the generator worker
    pub fn start(&'static self) {
        for g in self.gens.iter() {
            spawn(move || {
                g.cold_start();
                g.watch();
            });
        }
    }
}

struct ArticleGenerator;
struct GeneralGenerator;
struct StaticGenerator;

impl FileMapper for ArticleGenerator {
    fn src_root(&self) -> PathBuf {
        get_allay_config().content.dir.clone().into()
    }

    fn path_mapping(&self, src: &Path) -> PathBuf {
        let mut res = src.to_path_buf();
        if TemplateKind::from_filename(src) == TemplateKind::Markdown {
            res.set_extension(TemplateKind::Html.extension());
        }
        res
    }
}

fn write_with_wrapper(dest: PathBuf, html: &str) -> FileResult<()> {
    file::write_file(
        dest,
        &format!(
            include_str!("wrapper.html"),
            html,
            include_str!("auto-reload.js")
        ),
    )
}

macro_rules! file_generator_impl {
    ($generator: ident, $kind: expr) => {
        impl FileGenerator for $generator {
            fn created(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
                match COMPILER.lock().unwrap().compile_file(&src, $kind) {
                    Ok(output) => write_with_wrapper(dest, &output.html)?,
                    Err(e) => warn!("Failed to compile {:?}: {}", src, e),
                }
                Ok(())
            }

            fn removed(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
                COMPILER.lock().unwrap().remove_file(src.clone(), $kind);
                file::remove(dest)
            }

            fn modified(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
                COMPILER.lock().unwrap().modify_file(&src, $kind);
                match COMPILER.lock().unwrap().compile_file(&src, $kind) {
                    Ok(output) => write_with_wrapper(dest, &output.html)?,
                    Err(e) => warn!("Failed to compile {:?}: {}", src, e),
                }
                Ok(())
            }
        }
    };
}

file_generator_impl!(ArticleGenerator, ContentKind::Article);

impl FileMapper for GeneralGenerator {
    fn src_root(&self) -> PathBuf {
        get_theme_path()
            .join(&get_allay_config().theme.template.dir)
            .join(&get_allay_config().theme.template.custom_dir)
    }

    fn path_mapping(&self, src: &Path) -> PathBuf {
        let mut res = src.to_path_buf();
        if TemplateKind::from_filename(src) == TemplateKind::Markdown {
            res.set_extension(TemplateKind::Html.extension());
        }
        res
    }
}

file_generator_impl!(GeneralGenerator, ContentKind::General);

impl FileMapper for StaticGenerator {
    fn src_root(&self) -> PathBuf {
        get_allay_config().statics.dir.clone().into()
    }
}

impl FileGenerator for StaticGenerator {
    fn created(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        file::copy(src, dest)
    }

    fn removed(&self, _src: PathBuf, dest: PathBuf) -> FileResult<()> {
        file::remove(dest)
    }

    fn modified(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        file::remove(&dest)?;
        file::copy(src, dest)
    }
}
