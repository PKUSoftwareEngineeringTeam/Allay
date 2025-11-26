use crate::ast::{Meta, Template};
use crate::interpret::interpret_meta;
use crate::parse::parse_file;
use crate::{CompileError, CompileResult, magic};
use allay_base::config::get_allay_config;
use allay_base::data::{AllayData, AllayObject};
use allay_base::url::AllayUrlPath;
use allay_base::{file, template::TemplateKind};
#[cfg(feature = "plugin")]
use allay_plugin::PluginManager;
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, RwLock};

struct Cacher<T> {
    cache: HashMap<PathBuf, (u64, T)>,
}

impl<T> Cacher<T> {
    fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    fn get<P: AsRef<Path>>(&self, path: P, timestamp: u64) -> Option<&T> {
        let path = path.as_ref().to_path_buf();
        let (t, value) = self.cache.get(&path)?;
        if *t == timestamp { Some(value) } else { None }
    }

    fn insert<P: AsRef<Path>>(&mut self, path: P, timestamp: u64, value: T) -> Option<T> {
        let path = path.as_ref().to_path_buf();
        self.cache.insert(path, (timestamp, value)).map(|(_, v)| v)
    }
}

static AST_CACHER: LazyLock<RwLock<Cacher<Template>>> =
    LazyLock::new(|| RwLock::new(Cacher::new()));
static META_CACHER: LazyLock<RwLock<Cacher<AllayObject>>> =
    LazyLock::new(|| RwLock::new(Cacher::new()));

fn post_preprocess<P: AsRef<Path>>(source: P, mut meta: AllayObject) -> AllayObject {
    meta.entry(magic::URL.into()).or_insert_with(|| {
        // Add the `url` field to the metadata
        let entry =
            match source.as_ref().strip_prefix(file::workspace(&get_allay_config().content_dir)) {
                Ok(e) => e.with_extension(TemplateKind::Html.extension()),
                // ignore if the file is not under the content directory
                Err(_) => return Arc::new(AllayData::default()),
            };
        let url = AllayUrlPath::from(entry).as_ref().to_string_lossy().to_string();
        Arc::new(url.into())
    });
    meta
}

#[cfg(feature = "plugin")]
fn before_compile(content: String, kind: TemplateKind) -> String {
    let plugin_manager = PluginManager::instance();
    plugin_manager.plugins().iter().fold(content, |content, plugin| {
        let mut plugin = plugin.lock().expect("Plugin lock poisoned!");
        plugin.before_compile(content, kind.clone())
    })
}

pub fn get_meta_and_content<P: AsRef<Path>>(source: P) -> CompileResult<(AllayObject, Template)> {
    let kind = TemplateKind::from_filename(&source);
    if let TemplateKind::Other(e) = kind {
        return Err(CompileError::FileTypeNotSupported(e));
    }

    let last_modified = file::last_modified(&source)?;

    if let Some(ast) = AST_CACHER.read().unwrap().get(&source, last_modified)
        && let Some(meta) = META_CACHER.read().unwrap().get(&source, last_modified)
    {
        return Ok((meta.clone(), ast.clone()));
    }

    let content = file::read_file_string(&source)?;

    #[cfg(feature = "plugin")]
    let content = before_compile(content, kind);
    let ast = parse_file(&content)?;
    let mut meta = interpret_meta(&ast.meta)?;
    meta = post_preprocess(&source, meta);

    AST_CACHER.write().unwrap().insert(&source, last_modified, ast.template.clone());
    META_CACHER.write().unwrap().insert(source, last_modified, meta.clone());

    Ok((meta, ast.template))
}

pub fn get_meta<P: AsRef<Path>>(source: P) -> CompileResult<AllayObject> {
    let kind = TemplateKind::from_filename(&source);
    if let TemplateKind::Other(e) = kind {
        return Err(CompileError::FileTypeNotSupported(e));
    }

    let last_modified = file::last_modified(&source)?;

    if let Some(meta) = META_CACHER.read().unwrap().get(&source, last_modified) {
        return Ok(meta.clone());
    }

    let content = file::read_file_string(&source)?;

    #[cfg(feature = "plugin")]
    let content = before_compile(content, kind);

    static YAML_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)^---\s*(?P<yaml>.*?)\s*---").unwrap());
    static TOML_RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?s)^\+\+\+\s*(?P<toml>.*?)\s*\+\+\+").unwrap());

    let meta = if let Some(caps) = YAML_RE.captures(&content) {
        caps.name("yaml").map(|m| Meta::Yaml(m.as_str().into()))
    } else if let Some(caps) = TOML_RE.captures(&content) {
        caps.name("toml").map(|m| Meta::Toml(m.as_str().into()))
    } else {
        None
    };

    let meta = interpret_meta(&meta)?;
    let meta = post_preprocess(&source, meta);

    META_CACHER.write().unwrap().insert(source, last_modified, meta.clone());

    Ok(meta)
}

pub fn get_raw_content<P: AsRef<Path>>(source: P) -> CompileResult<String> {
    let kind = TemplateKind::from_filename(&source);
    let content = match kind {
        TemplateKind::Html | TemplateKind::Markdown => file::read_file_string(&source)?,
        TemplateKind::Other(e) => return Err(CompileError::FileTypeNotSupported(e)),
    };
    #[cfg(feature = "plugin")]
    let content = before_compile(content, kind);
    Ok(content)
}
