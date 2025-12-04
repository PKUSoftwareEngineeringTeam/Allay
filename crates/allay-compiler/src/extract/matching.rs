use crate::ast::{Meta, Template};
#[cfg(feature = "plugin")]
use crate::extract::process::before_compile;
use crate::extract::process::meta_preprocess;
use crate::interpret::interpret_meta;
use crate::parse::parse_file;
use crate::{CompileError, CompileResult};
use allay_base::data::AllayObject;
use allay_base::{file, template::TemplateKind};
use regex::Regex;
use std::path::Path;
use std::sync::{Arc, LazyLock};

/// Match and extract metadata and content from a source file
pub fn match_meta_and_content<P: AsRef<Path>>(
    source: P,
) -> CompileResult<(AllayObject, Arc<Template>)> {
    let kind = TemplateKind::from_filename(&source);
    if let TemplateKind::Other(e) = kind {
        return Err(CompileError::FileTypeNotSupported(e));
    }

    let content = file::read_file_string(&source)?;

    #[cfg(feature = "plugin")]
    let content = before_compile(content, kind);
    let ast = parse_file(&content)?;
    let mut meta = interpret_meta(&ast.meta)?;
    meta = meta_preprocess(&source, meta);
    let template = Arc::new(ast.template);

    Ok((meta, template))
}

/// Match and extract metadata from a source file only using regex
pub fn match_meta<P: AsRef<Path>>(source: P) -> CompileResult<AllayObject> {
    let kind = TemplateKind::from_filename(&source);
    if let TemplateKind::Other(e) = kind {
        return Err(CompileError::FileTypeNotSupported(e));
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
    let meta = meta_preprocess(&source, meta);

    Ok(meta)
}

/// Match and extract raw content from a source file
pub fn match_raw_content<P: AsRef<Path>>(source: P) -> CompileResult<String> {
    let kind = TemplateKind::from_filename(&source);
    let content = match kind {
        TemplateKind::Html | TemplateKind::Markdown => file::read_file_string(&source)?,
        TemplateKind::Other(e) => return Err(CompileError::FileTypeNotSupported(e)),
    };
    #[cfg(feature = "plugin")]
    let content = before_compile(content, kind);
    Ok(content)
}
