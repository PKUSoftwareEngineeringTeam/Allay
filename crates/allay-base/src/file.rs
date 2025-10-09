use serde::Serialize;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Error, Debug)]
pub enum FileError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),
}

pub type FileResult<T> = Result<T, FileError>;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileContent {
    pub path: PathBuf,
    pub content: String,
    pub size: usize,
    pub line_count: usize,
}

static ROOT: OnceLock<PathBuf> = OnceLock::new();

/// Set the root directory for the site
pub fn set_root<P: AsRef<Path>>(path: P) {
    if ROOT.get().is_some() {
        warn!("Root directory is already set. Ignoring subsequent set_root call.");
        return;
    }
    let path = path.as_ref().into();
    ROOT.set(path).ok();
}

pub fn absolute<P: AsRef<Path>>(path: P) -> PathBuf {
    env::current_dir().expect("Failed to get current directory").join(path)
}

/// Get the root directory for the site
pub fn root() -> PathBuf {
    if ROOT.get().is_none() {
        info!("Root directory is not set. Defaulting to current directory.");
        ROOT.set(".".into()).ok();
    }
    ROOT.get().unwrap().clone()
}

/// Get the absolute path of the root directory
pub fn absolute_root() -> PathBuf {
    absolute(root())
}

/// Get the actual workspace path by of the given path
pub fn workspace<P: AsRef<Path>>(path: P) -> PathBuf {
    root().join(path)
}

/// Get the actual absolute workspace path by of the given path
pub fn absolute_workspace<P: AsRef<Path>>(path: P) -> PathBuf {
    absolute_root().join(path)
}

/// Check if a file exists
pub fn file_exists<P: AsRef<Path>>(file_path: P) -> bool {
    let file_path = file_path.as_ref();
    file_path.exists() && file_path.is_file()
}

/// Check if a directory exists
pub fn dir_exists<P: AsRef<Path>>(dir_path: P) -> bool {
    let dir_path = dir_path.as_ref();
    dir_path.exists() && dir_path.is_dir()
}

/// Check if a directory is dirty (not empty)
pub fn dirty_dir<P: AsRef<Path>>(dir_path: P) -> FileResult<bool> {
    Ok(dir_exists(dir_path.as_ref()) && dir_path.as_ref().read_dir()?.next().is_some())
}

/// Read all the files in a directory recursively
pub fn read_dir_all_files<P: AsRef<Path>>(dir_path: P) -> FileResult<Vec<PathBuf>> {
    let dir_path = dir_path.as_ref();
    if !dir_exists(dir_path) {
        return Err(FileError::FileNotFound(dir_path.into()));
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir_path)? {
        let path = entry?.path();
        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            files.extend(read_dir_all_files(&path)?);
        }
    }
    Ok(files)
}

/// Read the metadata of a file
pub fn read_file_info<P: AsRef<Path>>(file_path: P) -> FileResult<fs::Metadata> {
    let file_path = file_path.as_ref();

    if !file_exists(file_path) {
        return Err(FileError::FileNotFound(file_path.into()));
    }

    let metadata = fs::metadata(file_path)?;
    Ok(metadata)
}

/// Read the entire content of a file
pub fn read_file<P: AsRef<Path>>(file_path: P) -> FileResult<FileContent> {
    let file_path = file_path.as_ref();

    if !file_exists(file_path) {
        return Err(FileError::FileNotFound(file_path.into()));
    }

    let content = fs::read_to_string(file_path)?;
    let line_count = content.lines().count();
    let size = content.len();

    Ok(FileContent {
        path: file_path.into(),
        content,
        size,
        line_count,
    })
}

/// Read the entire content of a file as only a string
pub fn read_file_string<P: AsRef<Path>>(file_path: P) -> FileResult<String> {
    let file_path = file_path.as_ref();

    if !file_exists(file_path) {
        return Err(FileError::FileNotFound(file_path.into()));
    }

    let content = fs::read_to_string(file_path)?;
    Ok(content)
}

/// Create a directory
pub fn create_dir<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    let dir_path = dir_path.as_ref();
    fs::create_dir(dir_path)?;
    Ok(())
}

/// Create a directory if it does not exist
pub fn create_dir_if_not_exists<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    if !dir_exists(&dir_path) {
        fs::create_dir(dir_path)?;
    }
    Ok(())
}

/// Create a directory and all its parent components if they are missing
pub fn create_dir_recursively<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    let dir_path = dir_path.as_ref();
    fs::create_dir_all(dir_path)?;
    Ok(())
}

/// Create an empty file
pub fn create_file<P: AsRef<Path>>(file_path: P) -> FileResult<()> {
    let file_path = file_path.as_ref();
    fs::File::create(file_path)?;
    Ok(())
}

/// Create an empty file, creating parent directories if necessary
pub fn create_file_recursively<P: AsRef<Path>>(file_path: P) -> FileResult<()> {
    let file_path = file_path.as_ref();
    if let Some(parent) = file_path.parent() {
        create_dir_recursively(parent)?;
    }
    fs::File::create(file_path)?;
    Ok(())
}

/// Write content to a file, creating parent directories if necessary
pub fn write_file<P: AsRef<Path>>(file_path: P, content: &str) -> FileResult<()> {
    let file_path = file_path.as_ref();
    if let Some(parent) = file_path.parent() {
        create_dir_recursively(parent)?;
    }
    fs::write(file_path, content)?;
    Ok(())
}

/// Remove a file or directory if it exists
pub fn remove<P: AsRef<Path>>(path: P) -> FileResult<()> {
    let path = path.as_ref();
    if path.exists() {
        if path.is_dir() {
            remove_dir_recursively(path)
        } else {
            remove_file(path)
        }
    } else {
        Ok(())
    }
}

/// Remove a file if it exists
pub fn remove_file<P: AsRef<Path>>(file_path: P) -> FileResult<()> {
    let file_path = file_path.as_ref();
    if file_exists(file_path) {
        fs::remove_file(file_path)?;
    }
    Ok(())
}

/// Remove an empty directory if it exists
pub fn remove_dir<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    let dir_path = dir_path.as_ref();
    if dir_exists(dir_path) {
        fs::remove_dir(dir_path)?;
    }
    Ok(())
}

/// Remove a directory and all its contents if it exists
pub fn remove_dir_recursively<P: AsRef<Path>>(dir_path: P) -> FileResult<()> {
    let dir_path = dir_path.as_ref();
    if dir_exists(dir_path) {
        fs::remove_dir_all(dir_path)?;
    }
    Ok(())
}

pub fn rename<P: AsRef<Path>>(old: P, new: P) -> FileResult<()> {
    if old.as_ref().exists() {
        fs::rename(old, new)?;
        Ok(())
    } else {
        Err(FileError::FileNotFound(old.as_ref().into()))
    }
}

pub fn copy<P: AsRef<Path>>(src: P, dest: P) -> FileResult<()> {
    if src.as_ref().exists() {
        fs::copy(src, dest)?;
        Ok(())
    } else {
        Err(FileError::FileNotFound(src.as_ref().into()))
    }
}
