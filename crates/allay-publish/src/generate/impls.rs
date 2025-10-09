use super::traits::{FileGenerator, FileMapper};
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

static FILEMAP: LazyLock<Mutex<HashMap<PathBuf, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn refresh(skip: &PathBuf) -> FileResult<()> {
    for (path, res) in COMPILER.lock().unwrap().refresh_pages(skip) {
        if let Some(dest) = FILEMAP.lock().unwrap().get(&path) {
            match res {
                Ok(output) => write_with_wrapper(dest, &output.html)?,
                Err(e) => warn!("Failed to recompile {:?}: {}", path, e),
            }
        }
        // otherwise it is not managed by this generator
    }
    Ok(())
}

macro_rules! file_generator_impl {
    ($generator: ident, $kind: expr) => {
        impl $generator {}

        impl FileGenerator for $generator {
            fn created(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
                FILEMAP.lock().unwrap().insert(src.clone(), dest.clone());
                match COMPILER.lock().unwrap().compile_file(&src, $kind) {
                    Ok(output) => write_with_wrapper(&dest, &output.html)?,
                    Err(e) => warn!("Failed to compile {:?}: {}", src, e),
                }
                refresh(&src)
            }

            fn removed(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
                FILEMAP.lock().unwrap().remove(&src);
                if let Err(e) = COMPILER.lock().unwrap().remove_file(src.clone(), $kind) {
                    warn!("Error when removing: {:?}: {}", src, e);
                }
                refresh(&src)?;
                file::remove(dest)
            }

            fn modified(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
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

impl FileMapper for GeneralGenerator {
    fn src_root(&self) -> PathBuf {
        get_theme_path()
            .join(&get_allay_config().theme.template.dir)
            .join(&get_allay_config().theme.template.custom_dir)
    }

    fn path_mapping(&self, src: &Path) -> PathBuf {
        let mut res = src.to_path_buf();
        if TemplateKind::from_filename(src).is_md() {
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
