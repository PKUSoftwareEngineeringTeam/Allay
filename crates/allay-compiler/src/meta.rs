use crate::ast::{Meta, Template};
use crate::interpret::interpret_meta;
use crate::parse::parse_file;
use crate::{CompileError, CompileResult, magic};
use allay_base::config::get_allay_config;
use allay_base::{data::AllayObject, file, template::TemplateKind};
#[cfg(feature = "plugin")]
use allay_plugin::PluginManager;
use regex::Regex;
use std::path::Path;
use std::sync::Arc;

fn post_preprocess<P: AsRef<Path>>(source: P, mut meta: AllayObject) -> AllayObject {
    meta.entry(magic::URL.into()).or_insert_with(|| {
        // Add the `url` field to the metadata
        let entry =
            match source.as_ref().strip_prefix(file::workspace(&get_allay_config().content_dir)) {
                Ok(e) => e,
                // ignore if the file is not under the content directory
                Err(_) => return Arc::new(().into()),
            };
        // for "foo\\bar.html", we change it to "foo/bar.html"
        let url = Path::new("")
            .join(entry)
            .with_extension(TemplateKind::Html.extension())
            .to_string_lossy()
            .to_string()
            .replace('\\', "/") // for Windows paths
            .into();
        Arc::new(url)
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
    let content = match kind {
        TemplateKind::Html | TemplateKind::Markdown => file::read_file_string(&source)?,
        TemplateKind::Other(e) => return Err(CompileError::FileTypeNotSupported(e)),
    };
    #[cfg(feature = "plugin")]
    let content = before_compile(content, kind);
    let ast = parse_file(&content)?;
    let mut meta = interpret_meta(&ast.meta)?;
    meta = post_preprocess(source, meta);

    Ok((meta, ast.template))
}

pub fn get_meta<P: AsRef<Path>>(source: P) -> CompileResult<AllayObject> {
    let kind = TemplateKind::from_filename(&source);
    let content = match kind {
        TemplateKind::Html | TemplateKind::Markdown => file::read_file_string(&source)?,
        TemplateKind::Other(e) => return Err(CompileError::FileTypeNotSupported(e)),
    };

    #[cfg(feature = "plugin")]
    let content = before_compile(content, kind);

    // find the meta without parsing the whole template
    let yaml_re = Regex::new(r"(?s)^---\s*(?P<yaml>.*?)\s*---").unwrap();
    let toml_re = Regex::new(r"(?s)^\+\+\+\s*(?P<toml>.*?)\s*\+\+\+").unwrap();

    let meta = if let Some(caps) = yaml_re.captures(&content) {
        caps.name("yaml").map(|m| Meta::Yaml(m.as_str().into()))
    } else if let Some(caps) = toml_re.captures(&content) {
        caps.name("toml").map(|m| Meta::Toml(m.as_str().into()))
    } else {
        None
    };

    let mut meta = interpret_meta(&meta)?;
    meta = post_preprocess(source, meta);

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
