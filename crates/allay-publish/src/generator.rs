use allay_base::config::{CLICommand, get_cli_config};
use allay_base::file::{self, FileResult};
use allay_base::template::{FileKind, TemplateKind};
use allay_compiler::Compiler;
use notify::event::{EventKind, ModifyKind, RenameMode};
use notify_debouncer_full::{DebounceEventResult, DebouncedEvent, new_debouncer};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{LazyLock, Mutex};
use std::thread;
use std::time::Duration;
use tracing::{info, warn};
use walkdir::WalkDir;

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

    /// and triggering the `on_create` event for each file except those that satisfy the `skip` condition.
    fn cold_start(&self) {
        let root = file::absolute_workspace(self.root());
        for entry in WalkDir::new(&root).follow_links(true) {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        let path = self.to_relative(entry.path());
                        self.on_create(path.clone()).unwrap_or_else(|e| {
                            warn!("Error handling cold start file {:?}: {}", path, e);
                        });
                    }
                }
                Err(e) => {
                    warn!("Error reading file in cold start in {:?}: {}", root, e);
                    continue;
                }
            };
        }
    }

    /// Start watching the root directory for file events.
    fn watch(&self) {
        let root = file::workspace(self.root());
        let (tx, rx) = mpsc::channel();

        let debouncer = new_debouncer(Duration::from_millis(50), None, tx);

        let mut debouncer = match debouncer {
            Ok(debouncer) => debouncer,
            Err(e) => {
                return warn!("Failed to create file watcher in {:?}: {}", root, e);
            }
        };

        if let Err(e) = debouncer.watch(root.clone(), notify::RecursiveMode::Recursive) {
            warn!("Failed to watch directory {:?}: {}", root, e);
        }

        while let Ok(event) = rx.recv() {
            self.notify_event_handler(event).unwrap_or_else(|e| {
                warn!("Error handling file event in {:?}: {}", root, e);
            });
        }
        info!("File watcher channel in {:?} closed!", root);
    }

    /// Start listening to file events in a new thread.
    fn start_listening(&'static self) {
        thread::spawn(move || {
            self.cold_start();
            self.watch();
        });
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
    fn to_relative(&self, path: &Path) -> PathBuf {
        let root = file::absolute_workspace(self.root());
        path.strip_prefix(root).unwrap_or(path).into()
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
    fn path_mapping(&self, src: &Path) -> PathBuf {
        src.into()
    }

    /// Utility function to get the source path in the workspace.
    /// Do not override this function unless necessary.
    fn src_workspace(&self, src: &Path) -> PathBuf {
        file::workspace(self.src_root()).join(src)
    }

    /// Utility function to get the destination path in the workspace.
    /// Do not override this function unless necessary.
    fn dest_workspace(&self, src: &Path) -> PathBuf {
        file::workspace(self.dest_root()).join(self.path_mapping(src))
    }
}

/// A struct that combines [`FileListener`] and [`FileMapper`] to provide
/// file generating capabilities from a source directory to a destination directory.
/// Note: all path parameters here are both the path relative to the workspace root.
pub struct FileGenerator {
    options: FileGeneratorOptions,
}

/// Options for the [`FileGenerator`].
#[derive(Default)]
pub struct FileGeneratorOptions {
    src_root: PathBuf,
    dest_root: PathBuf,
    kind: FileKind,
    map_to_html: bool,
}

/// Global compiler instance for all file generators
static COMPILER: LazyLock<Mutex<Compiler<String>>> =
    LazyLock::new(|| Mutex::new(Compiler::default()));

/// A global file mapping from source path to destination path
static FILE_MAP: LazyLock<Mutex<HashMap<PathBuf, PathBuf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl FileGenerator {
    /// Create a new file generator.
    pub fn new(options: FileGeneratorOptions) -> Self {
        Self { options }
    }

    /// Determine whether the file should not be compiled.
    fn no_compile(&self, src: &PathBuf) -> bool {
        matches!(self.options.kind, FileKind::Static)
            || !TemplateKind::from_filename(src).is_template()
    }

    /// What to do when a file is created.
    fn created(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        if matches!(self.options.kind, FileKind::Wrapper) {
            return Ok(()); // wrapper files are not generated directly
        }
        if self.no_compile(&src) {
            return file::copy(src, dest);
        }

        FILE_MAP.lock().unwrap().insert(src.clone(), dest.clone());

        match COMPILER.lock().unwrap().compile_file(&src, &self.options.kind) {
            Ok(output) => Self::write_with_wrapper(&dest, &output.html)?,
            Err(e) => warn!("Failed to compile {:?}: {}", src, e),
        }
        Self::refresh()
    }

    /// What to do when a file is removed.
    fn removed(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        if self.no_compile(&src) {
            return file::remove(dest);
        }

        COMPILER.lock().unwrap().remove(&src);
        if matches!(&self.options.kind, FileKind::Wrapper) {
            return Self::refresh();
        }

        FILE_MAP.lock().unwrap().remove(&src);
        Self::refresh()?;
        file::remove(dest)
    }

    /// What to do when a file is modified.
    fn modified(&self, src: PathBuf, dest: PathBuf) -> FileResult<()> {
        if self.no_compile(&src) {
            return file::copy(src, dest);
        }
        COMPILER.lock().unwrap().modify(&src);
        if matches!(self.options.kind, FileKind::Wrapper) {
            return Self::refresh();
        }
        match COMPILER.lock().unwrap().compile_file(&src, &self.options.kind) {
            Ok(output) => Self::write_with_wrapper(&dest, &output.html)?,
            Err(e) => warn!("Failed to compile {:?}: {}", src, e),
        }
        Self::refresh()
    }

    fn write_with_wrapper(dest: &PathBuf, html: &str) -> FileResult<()> {
        let hot_reload = matches!(get_cli_config().command, CLICommand::Serve(_))
            .then_some(include_str!("assets/auto-reload.js"))
            .unwrap_or_default();
        file::write_file(
            dest,
            &format!(include_str!("assets/wrapper.html"), html, hot_reload),
        )
    }

    /// handling the recompilation of all affected files
    fn refresh() -> FileResult<()> {
        let pages = COMPILER.lock().unwrap().refresh_pages();
        for (path, res) in pages {
            if let Some(dest) = FILE_MAP.lock().unwrap().get(&path) {
                match res {
                    Ok(output) => Self::write_with_wrapper(dest, &output.html)?,
                    Err(e) => warn!("Failed to recompile {:?}: {}", path, e),
                }
            }
        }
        Ok(())
    }
}

impl FileMapper for FileGenerator {
    fn src_root(&self) -> PathBuf {
        self.options.src_root.clone()
    }

    fn dest_root(&self) -> PathBuf {
        self.options.dest_root.clone()
    }

    fn path_mapping(&self, src: &Path) -> PathBuf {
        if self.options.map_to_html {
            let mut res = src.to_path_buf();
            if TemplateKind::from_filename(src).is_md() {
                res.set_extension(TemplateKind::Html.extension());
            }
            res
        } else {
            src.into()
        }
    }
}

impl FileListener for FileGenerator {
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

impl FileGeneratorOptions {
    pub fn src_root(mut self, src_root: PathBuf) -> Self {
        self.src_root = src_root;
        self
    }

    pub fn dest_root(mut self, dest_root: PathBuf) -> Self {
        self.dest_root = dest_root;
        self
    }

    pub fn kind(mut self, kind: FileKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn map_to_html(mut self, to_html: bool) -> Self {
        self.map_to_html = to_html;
        self
    }

    pub fn build(self) -> FileGenerator {
        FileGenerator::new(self)
    }
}
