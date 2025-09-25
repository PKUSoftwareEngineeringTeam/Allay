use allay_base::file::{self, FileResult};
use notify::event::{EventKind, ModifyKind, RenameMode};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent, new_debouncer};
use std::{path::PathBuf, sync::mpsc, time::Duration};
use tracing::{info, warn};

/// A trait for listening to file system events in a specified root directory.
pub trait FileListener: Send + Sync {
    /// The root directory to listen to.
    fn root(&self) -> PathBuf;

    /// The event handler on file creation.
    fn on_create(&self, path: PathBuf) -> FileResult<()>;

    /// The event handler on file removal.
    fn on_remove(&self, path: PathBuf) -> FileResult<()>;

    /// The event handler on file modification.
    fn on_modify(&self, path: PathBuf) -> FileResult<()> {
        self.on_remove(path.clone())?;
        self.on_create(path)
    }

    /// The event handler on file renaming.
    fn on_rename(&self, old: PathBuf, new: PathBuf) -> FileResult<()> {
        self.on_remove(old)?;
        self.on_create(new)
    }

    /// Start watching the root directory for file events.
    fn watch(&'static self) {
        let root = file::workspace(self.root());
        let (tx, rx) = mpsc::channel();

        let debouncer = new_debouncer(Duration::from_secs(1), None, tx);

        let mut debouncer = match debouncer {
            Ok(debouncer) => debouncer,
            Err(e) => {
                return warn!("Failed to create file watcher: {}", e);
            }
        };

        if let Err(e) = debouncer.watch(root.clone(), notify::RecursiveMode::Recursive) {
            warn!("Failed to watch directory: {}", e);
        }

        while let Ok(event) = rx.recv() {
            self.notify_event_handler(event).unwrap_or_else(|e| {
                warn!("Error handling file event: {}", e);
            });
        }
        info!("File watcher channel closed!");
    }

    /// The main event handler to be called by the file watcher.
    fn notify_event_handler(&self, event: DebounceEventResult) -> FileResult<()> {
        if let Ok(events) = event {
            for event in events {
                self.on_notify_event(&event)?;
            }
        }
        Ok(())
    }

    /// Convert an absolute path provided by [`notify`] to a path relative to the root directory.
    /// Do not override this function unless necessary.
    fn to_relative(&self, path: &PathBuf) -> PathBuf {
        let root = file::absolute_workspace(self.root());
        path.strip_prefix(root).unwrap_or(path).to_path_buf()
    }

    /// An implementation detail for handling a single notify event.
    /// Do not override this function unless necessary.
    fn on_notify_event(&self, event: &DebouncedEvent) -> FileResult<()> {
        let paths = &event.paths;
        let path = self.to_relative(&paths[0]);
        match &event.kind {
            EventKind::Create(_) => self.on_create(path)?,
            EventKind::Modify(modify) => {
                match modify {
                    ModifyKind::Name(name) => {
                        match name {
                            RenameMode::Both => {
                                self.on_rename(path, self.to_relative(&paths[1]))?
                            }
                            // usually happen because of moving file to outside
                            RenameMode::From => self.on_remove(path)?,
                            // usually happen because of moving file from outside
                            RenameMode::To => self.on_create(path)?,
                            _ => {}
                        }
                    }
                    _ => self.on_modify(path)?,
                }
            }
            EventKind::Remove(_) => self.on_remove(path)?,
            _ => {}
        }
        Ok(())
    }
}

/// A trait for mapping files from a source workspace to a destination workspace.
pub trait FileMapper {
    /// The root directory of the source files.
    fn src_root(&self) -> PathBuf;

    /// The root directory of the destination files.
    fn dest_root(&self) -> PathBuf;

    /// The rule to map the path from source to destination.
    /// Note: the path parameters are the paths relative to the respective roots.
    /// Default: identity mapping
    fn path_mapping(&self, src: &PathBuf) -> PathBuf {
        src.clone()
    }

    /// Utility function to get the source path in the workspace.
    /// Do not override this function unless necessary.
    fn src_workspace(&self, src: &PathBuf) -> PathBuf {
        file::workspace(self.src_root()).join(src)
    }

    /// Utility function to get the destination path in the workspace.
    /// Do not override this function unless necessary.
    fn dest_workspace(&self, src: &PathBuf) -> PathBuf {
        file::workspace(self.dest_root()).join(self.path_mapping(src))
    }
}

/// A trait that combines [`FileListener`] and [`FileMapper`] to provide
/// file generating capabilities from a source directory to a destination directory.
/// Note: all path parameters here are both the path relative to the workspace root.
pub trait FileGenerator: FileListener + FileMapper {
    /// What to do when a file is created.
    fn created(&self, src: PathBuf, dest: PathBuf) -> FileResult<()>;

    /// What to do when a file is removed.
    fn removed(&self, src: PathBuf, dest: PathBuf) -> FileResult<()>;

    /// What to do when a file is modified.
    fn modified(&self, src: PathBuf, dest: PathBuf) -> FileResult<()>;
}

impl<T: FileGenerator> FileListener for T {
    fn root(&self) -> PathBuf {
        self.src_root()
    }

    fn on_create(&self, path: PathBuf) -> FileResult<()> {
        let src = self.src_workspace(&path);
        let dest = self.dest_workspace(&path);
        if src.is_dir() {
            file::create_dir_if_not_exists(dest)
        } else {
            self.created(src, dest)
        }
    }

    fn on_remove(&self, path: PathBuf) -> FileResult<()> {
        let src = self.src_workspace(&path);
        let dest = self.dest_workspace(&path);
        if src.is_dir() {
            file::remove_dir_recursively(&dest)
        } else {
            self.removed(src, dest)
        }
    }

    fn on_modify(&self, path: PathBuf) -> FileResult<()> {
        // only modify the dest file if the source file exists
        let src = self.src_workspace(&path);
        if src.is_file() {
            let dest = self.dest_workspace(&path);
            self.modified(src, dest)?;
        }
        Ok(())
    }
}
