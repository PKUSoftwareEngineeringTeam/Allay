use serde::Serialize;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
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

pub type FileResult<T> = std::result::Result<T, FileError>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub size: u64,
    pub extension: Option<String>,
    pub modified: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileContent {
    pub path: PathBuf,
    pub content: String,
    pub size: usize,
    pub line_count: usize,
}

pub struct FileUtils;

impl FileUtils {
    pub fn absolute_path<P: AsRef<Path>>(path: P) -> FileResult<PathBuf> {
        let path = path.as_ref();
        if path.is_absolute() {
            return Ok(path.to_path_buf());
        }
        let current_dir = std::env::current_dir()?;
        Ok(current_dir.join(path))
    }

    pub fn walk_dir<P: AsRef<Path>>(dir_path: P) -> FileResult<Vec<FileInfo>> {
        let dir_path = dir_path.as_ref();
        let mut file_infos = Vec::new();

        for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
            let metadata = entry.metadata()?;
            if !metadata.is_file() {
                continue;
            }

            let relative_path = entry
                .path()
                .strip_prefix(dir_path)
                .map_err(|_| FileError::InvalidUtf8Path(entry.path().to_path_buf()))?;

            let extension = entry
                .path()
                .extension()
                .and_then(OsStr::to_str)
                .map(|s| s.to_string());

            file_infos.push(FileInfo {
                path: entry.path().to_path_buf(),
                relative_path: relative_path.to_path_buf(),
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

    pub fn write_file<P: AsRef<Path>>(file_path: P, content: &str) -> FileResult<()> {
        let file_path = file_path.as_ref();
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)?;
        Ok(())
    }
}
