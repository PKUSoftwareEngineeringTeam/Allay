use serde::Serialize;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use thiserror::Error;
use tracing::warn;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid UTF-8 in file path: {0}")]
    InvalidUtf8Path(PathBuf),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),
}

pub type FileResult<T> = Result<T, FileError>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileInfo {
    pub root: PathBuf,
    pub path: PathBuf,
    pub size: u64,
    pub extension: Option<String>,
    pub modified: Option<std::time::SystemTime>,
}

impl FileInfo {
    pub fn relative_path(&self) -> PathBuf {
        self.path.strip_prefix(&self.root).unwrap_or(&self.path).to_path_buf()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileContent {
    pub path: PathBuf,
    pub content: String,
    pub size: usize,
    pub line_count: usize,
}

static ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn set_root<P: AsRef<Path>>(path: P) {
    if ROOT.get().is_some() {
        warn!("Root directory is already set. Ignoring subsequent set_root call.");
        return;
    }
    let path = path.as_ref().to_path_buf();
    ROOT.set(path).ok();
}

pub fn root() -> PathBuf {
    if ROOT.get().is_none() {
        ROOT.set(".".into()).ok();
    }
    ROOT.get().unwrap().clone()
}

pub fn workspace<P: AsRef<Path>>(path: P) -> PathBuf {
    if ROOT.get().is_none() {
        ROOT.set(".".into()).ok();
    }
    ROOT.get().unwrap().join(path)
}

pub fn walk_dir<P: AsRef<Path>>(dir_path: P) -> FileResult<Vec<FileInfo>> {
    let dir_path = dir_path.as_ref();
    let mut file_infos = Vec::new();

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        let metadata = entry.metadata()?;
        if !metadata.is_file() {
            continue;
        }

        let extension = entry.path().extension().and_then(OsStr::to_str).map(|s| s.to_string());

        file_infos.push(FileInfo {
            root: dir_path.to_path_buf(),
            path: entry.path().to_path_buf(),
            size: metadata.len(),
            extension,
            modified: metadata.modified().ok(),
        });
    }

    Ok(file_infos)
}

pub fn read_file<P: AsRef<Path>>(file_path: P) -> FileResult<FileContent> {
    let file_path = file_path.as_ref();

    if !file_path.exists() {
        return Err(FileError::FileNotFound(file_path.to_path_buf()));
    }

    if !file_path.is_file() {
        return Err(FileError::UnsupportedFileType(
            "Path is not a file".to_string(),
        ));
    }

    let content = fs::read_to_string(file_path)?;
    let line_count = content.lines().count();
    let size = content.len();

    Ok(FileContent {
        path: file_path.to_path_buf(),
        content,
        size,
        line_count,
    })
}

pub fn dirty_dir<P: AsRef<Path>>(dir_path: P) -> FileResult<bool> {
    let dir = dir_path.as_ref();
    Ok(dir.exists() && dir.is_dir() && dir.read_dir()?.next().is_some())
}

pub fn create_dir<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    let dir_path = dir_path.as_ref();
    fs::create_dir(dir_path)?;
    Ok(())
}

pub fn create_dir_if_not_exists<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    let dir_path = dir_path.as_ref();
    if !dir_path.exists() {
        fs::create_dir(dir_path)?;
    }
    Ok(())
}

pub fn create_dir_recursively<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    let dir_path = dir_path.as_ref();
    fs::create_dir_all(dir_path)?;
    Ok(())
}

pub fn create_file<P: AsRef<Path>>(file_path: P) -> FileResult<()> {
    let file_path = file_path.as_ref();
    fs::File::create(file_path)?;
    Ok(())
}

pub fn create_file_recursively<P: AsRef<Path>>(file_path: P) -> FileResult<()> {
    let file_path = file_path.as_ref();
    if let Some(parent) = file_path.parent() {
        create_dir_recursively(parent)?;
    }
    fs::File::create(file_path)?;
    Ok(())
}

pub fn write_file<P: AsRef<Path>>(file_path: P, content: &str) -> FileResult<()> {
    let file_path = file_path.as_ref();
    if let Some(parent) = file_path.parent() {
        create_dir_recursively(parent)?;
    }
    fs::write(file_path, content)?;
    Ok(())
}

pub fn remove_file<P: AsRef<Path>>(file_path: P) -> FileResult<()> {
    let file_path = file_path.as_ref();
    if file_path.exists() && file_path.is_file() {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

pub fn remove_dir<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    let dir_path = dir_path.as_ref();
    if dir_path.exists() && dir_path.is_dir() {
        fs::remove_dir(dir_path)?;
    }
    Ok(())
}

pub fn remove_dir_recursively<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    let dir_path = dir_path.as_ref();
    if dir_path.exists() && dir_path.is_dir() {
        fs::remove_dir_all(dir_path)?;
    }
    Ok(())
}
