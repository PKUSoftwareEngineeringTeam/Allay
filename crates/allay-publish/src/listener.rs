use allay_base::file::{self, FileResult};
use notify::event::{EventKind, ModifyKind, RenameMode};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent};
use std::path::{Path, PathBuf};

/// A trait for listening to file system events in a specified root directory.
pub trait FileListener {
    /// The root directory to listen to.
    fn root() -> String;

    /// The event handler on file creation.
    fn on_create<P: AsRef<Path>>(path: P) -> FileResult<()>;

    /// The event handler on file removal.
    fn on_remove<P: AsRef<Path>>(path: P) -> FileResult<()>;

    /// The event handler on file modification.
    fn on_modify<P: AsRef<Path>>(path: P) -> FileResult<()> {
        Self::on_remove(&path)?;
        Self::on_create(&path)
    }

    /// The event handler on file renaming.
    fn on_rename<P: AsRef<Path>>(old: P, new: P) -> FileResult<()> {
        Self::on_remove(old)?;
        Self::on_create(new)
    }

    /// The main event handler to be called by the file watcher.
    fn notify_event_handler(event: DebounceEventResult) -> FileResult<()> {
        if let Ok(events) = event {
            for event in events {
                Self::on_notify_event(&event)?;
            }
        }
        Ok(())
    }

    /// An implementation detail for handling a single notify event.
    /// Do not override this function unless necessary.
    fn on_notify_event(event: &DebouncedEvent) -> FileResult<()> {
        let paths = &event.paths;
        let path = &paths[0];
        match &event.kind {
            EventKind::Create(_) => Self::on_create(path)?,
            EventKind::Modify(modify) => {
                match modify {
                    ModifyKind::Name(name) => {
                        match name {
                            RenameMode::Both => Self::on_rename(path, &paths[1])?,
                            // usually happen because of moving file to outside
                            RenameMode::From => Self::on_remove(path)?,
                            // usually happen because of moving file from outside
                            RenameMode::To => Self::on_create(path)?,
                            _ => {}
                        }
                    }
                    _ => Self::on_modify(path)?,
                }
            }
            EventKind::Remove(_) => Self::on_remove(path)?,
            _ => {}
        }
        Ok(())
    }
}

/// A trait for mapping files from a source workspace to a destination workspace.
pub trait FileMapper {
    /// The root directory of the source files.
    fn source_root() -> String;

    /// The root directory of the destination files.
    fn dest_root() -> String;

    /// The rule to map the path from source to destination.
    /// Default: identity mapping
    fn path_mapping<P: AsRef<Path>>(from: P) -> PathBuf {
        from.as_ref().to_path_buf()
    }

    /// Utility function to get the source path in the workspace.
    /// Do not override this function unless necessary.
    fn path_source<P: AsRef<Path>>(from: P) -> PathBuf {
        file::workspace(Self::source_root()).join(from)
    }

    /// Utility function to get the destination path in the workspace.
    /// Do not override this function unless necessary.
    fn path_dest<P: AsRef<Path>>(from: P) -> PathBuf {
        file::workspace(Self::dest_root()).join(Self::path_mapping(from))
    }
}

/// A trait that combines [`FileListener`] and [`FileMapper`] to provide
/// file publishing capabilities from a source directory to a destination directory.
pub trait FilePublisher: FileListener + FileMapper {
    /// The publishing rule from source to destination.
    /// Default: copies the file from source to destination.
    fn publish<P: AsRef<Path>>(source: P, dest: P) -> FileResult<()> {
        let source = Self::path_source(&source);
        let dest = Self::path_dest(&dest);
        file::remove(&dest)?;
        file::copy(source, dest)
    }
}

impl<T: FilePublisher> FileListener for T {
    fn root() -> String {
        Self::source_root()
    }

    fn on_create<P: AsRef<Path>>(path: P) -> FileResult<()> {
        let source = Self::path_source(&path);
        let dest = Self::path_dest(&path);
        if source.is_dir() {
            file::create_dir_if_not_exists(dest)
        } else {
            file::create_file(dest)
        }
    }

    fn on_remove<P: AsRef<Path>>(path: P) -> FileResult<()> {
        let dest = Self::path_dest(&path);
        file::remove(&dest)
    }

    fn on_rename<P: AsRef<Path>>(old: P, new: P) -> FileResult<()> {
        // only rename the dest file if the source file exists
        let new_dest = Self::path_dest(&new);
        file::remove(&new_dest)?;
        file::rename(Self::path_dest(&old), new_dest)
    }

    fn on_modify<P: AsRef<Path>>(path: P) -> FileResult<()> {
        // only modify the dest file if the source file exists
        let source = Self::path_source(&path);
        if source.is_file() {
            let dest = Self::path_dest(&path);
            Self::publish(source, dest)?;
        }
        Ok(())
    }
}
