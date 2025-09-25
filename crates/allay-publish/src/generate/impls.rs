use super::{FileGenerator, FileMapper};
use allay_base::config::get_allay_config;
use allay_base::file::{self, FileResult};
use std::path::PathBuf;

#[derive(Default)]
pub struct Generators {
    gens: Vec<Box<dyn FileGenerator>>,
}

struct ArticleGenerator;
struct GeneralGenerator;
struct StaticGenerator;

impl FileMapper for ArticleGenerator {
    fn src_root(&self) -> String {
        get_allay_config().content.dir.clone()
    }

    fn dest_root(&self) -> String {
        get_allay_config().publish.dir.clone()
    }
}

impl FileGenerator for ArticleGenerator {
    fn created(&mut self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        file::copy(src, dest)
    }

    fn removed(&mut self, _src: PathBuf, dest: PathBuf) -> FileResult<()> {
        file::remove(dest)
    }

    fn modified(&mut self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        file::remove(&dest)?;
        file::copy(src, dest)
    }
}

impl FileMapper for StaticGenerator {
    fn src_root(&self) -> String {
        get_allay_config().statics.dir.clone()
    }

    fn dest_root(&self) -> String {
        get_allay_config().publish.dir.clone()
    }
}

impl FileGenerator for StaticGenerator {
    fn created(&mut self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        file::copy(src, dest)
    }

    fn removed(&mut self, _src: PathBuf, dest: PathBuf) -> FileResult<()> {
        file::remove(dest)
    }

    fn modified(&mut self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        file::remove(&dest)?;
        file::copy(src, dest)
    }
}
