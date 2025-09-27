use super::traits::{FileGenerator, FileMapper};
use allay_base::config::{get_allay_config, get_theme_path};
use allay_base::file::{self, FileResult};
use allay_base::template::ContentKind;
use allay_compiler::Compiler;
use std::path::PathBuf;
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

    fn dest_root(&self) -> PathBuf {
        get_allay_config().publish.dir.clone().into()
    }
}

impl FileGenerator for ArticleGenerator {
    fn created(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        match COMPILER.lock().unwrap().compile_file(&src, ContentKind::Article) {
            Ok(html) => file::write_file(dest, &html)?,
            Err(e) => warn!("Failed to compile {:?}: {}", src, e),
        }
        Ok(())
    }

    fn removed(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        COMPILER.lock().unwrap().remove_file(src.clone(), ContentKind::Article);
        file::remove(dest)
    }

    fn modified(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        COMPILER.lock().unwrap().modify_file(&src, ContentKind::Article);
        match COMPILER.lock().unwrap().compile_file(&src, ContentKind::Article) {
            Ok(html) => file::write_file(dest, &html)?,
            Err(e) => warn!("Failed to compile {:?}: {}", src, e),
        }
        Ok(())
    }
}

impl FileMapper for GeneralGenerator {
    fn src_root(&self) -> PathBuf {
        get_theme_path()
            .join(&get_allay_config().theme.template.dir)
            .join(&get_allay_config().theme.template.custom_dir)
    }

    fn dest_root(&self) -> PathBuf {
        get_allay_config().publish.dir.clone().into()
    }
}

impl FileGenerator for GeneralGenerator {
    fn created(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        match COMPILER.lock().unwrap().compile_file(&src, ContentKind::General) {
            Ok(html) => file::write_file(dest, &html)?,
            Err(e) => warn!("Failed to compile {:?}: {}", src, e),
        }
        Ok(())
    }

    fn removed(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        COMPILER.lock().unwrap().remove_file(src.clone(), ContentKind::General);
        file::remove(dest)
    }

    fn modified(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        COMPILER.lock().unwrap().modify_file(&src, ContentKind::General);
        match COMPILER.lock().unwrap().compile_file(&src, ContentKind::General) {
            Ok(html) => file::write_file(dest, &html)?,
            Err(e) => warn!("Failed to compile {:?}: {}", src, e),
        }
        Ok(())
    }
}

impl FileMapper for StaticGenerator {
    fn src_root(&self) -> PathBuf {
        get_allay_config().statics.dir.clone().into()
    }

    fn dest_root(&self) -> PathBuf {
        get_allay_config().publish.dir.clone().into()
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
