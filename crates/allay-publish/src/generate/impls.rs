use super::traits::{FileGenerator, FileMapper};
use allay_base::config::theme::get_theme_config;
use allay_base::config::{CLICommand, get_allay_config, get_cli_config, get_theme_path};
use allay_base::file::{self, FileResult};
use allay_base::template::{ContentKind, TemplateKind};
use allay_compiler::Compiler;
use std::collections::HashMap;
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
                Box::new(ThemeStaticGenerator),
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
struct ThemeStaticGenerator;
struct ArticleGenerator;
struct GeneralGenerator;

impl FileMapper for StaticGenerator {
    fn src_root(&self) -> PathBuf {
        get_allay_config().statics_dir.clone().into()
    }
}

impl FileGenerator for StaticGenerator {}

impl FileMapper for ThemeStaticGenerator {
    fn src_root(&self) -> PathBuf {
        get_theme_path().join(&get_theme_config().config.static_dir)
    }
}

impl FileGenerator for ThemeStaticGenerator {}

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

impl FileMapper for GeneralGenerator {
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

fn write_with_wrapper(dest: &PathBuf, html: &str) -> FileResult<()> {
    let hot_reload = matches!(get_cli_config().command, CLICommand::Serve(_))
        .then_some(include_str!("auto-reload.js"))
        .unwrap_or_default();
    file::write_file(
        dest,
        &format!(include_str!("wrapper.html"), html, hot_reload),
    )
}

/// A global file mapping from source path to destination path
static FILE_MAP: LazyLock<Mutex<HashMap<PathBuf, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// handling the recompilation of all affected files
fn refresh(skip: &PathBuf) -> FileResult<()> {
    for (path, res) in COMPILER.lock().unwrap().refresh_pages(skip) {
        if let Some(dest) = FILE_MAP.lock().unwrap().get(&path) {
            match res {
                Ok(output) => write_with_wrapper(dest, &output.html)?,
                Err(e) => warn!("Failed to recompile {:?}: {}", path, e),
            }
        }
    }
    Ok(())
}

macro_rules! file_generator_impl {
    ($generator: ident, $kind: expr) => {
        impl FileGenerator for $generator {
            fn created(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
                if !TemplateKind::from_filename(&src).is_template() {
                    return FileGenerator::created(self, src, dest);
                }

                FILE_MAP.lock().unwrap().insert(src.clone(), dest.clone());
                match COMPILER.lock().unwrap().compile_file(&src, $kind) {
                    Ok(output) => write_with_wrapper(&dest, &output.html)?,
                    Err(e) => warn!("Failed to compile {:?}: {}", src, e),
                }
                refresh(&src)
            }

            fn removed(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
                if !TemplateKind::from_filename(&src).is_template() {
                    return FileGenerator::removed(self, src, dest);
                }

                FILE_MAP.lock().unwrap().remove(&src);
                if let Err(e) = COMPILER.lock().unwrap().remove_file(src.clone(), $kind) {
                    warn!("Error when removing: {:?}: {}", src, e);
                }
                refresh(&src)?;
                file::remove(dest)
            }

            fn modified(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
                if !TemplateKind::from_filename(&src).is_template() {
                    return FileGenerator::modified(self, src, dest);
                }

                if let Err(e) = COMPILER.lock().unwrap().modify_file(&src, $kind) {
                    warn!("Error when modifying: {:?}: {}", src, e);
                }
                match COMPILER.lock().unwrap().compile_file(&src, $kind) {
                    Ok(output) => write_with_wrapper(&dest, &output.html)?,
                    Err(e) => warn!("Failed to compile {:?}: {}", src, e),
                }
                refresh(&src)
            }
        }
    };
}

file_generator_impl!(ArticleGenerator, ContentKind::Article);
file_generator_impl!(GeneralGenerator, ContentKind::General);
