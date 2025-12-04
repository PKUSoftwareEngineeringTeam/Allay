mod cache;
mod matching;
mod process;

use crate::CompileResult;
use crate::ast::Template;
use allay_base::data::AllayObject;
use allay_base::file;
use allay_base::sitemap::SiteMap;
use cache::FileCacher;
pub use matching::*;
use pulldown_cmark::{Options, Parser, html};
use std::path::Path;
use std::sync::{Arc, LazyLock, RwLock};

static AST_CACHER: LazyLock<RwLock<FileCacher<Arc<Template>>>> =
    LazyLock::new(|| RwLock::new(FileCacher::new()));

/// Get metadata and content template from a source file, using cache if available
pub fn get_meta_and_content<P: AsRef<Path>>(
    source: P,
) -> CompileResult<(AllayObject, Arc<Template>)> {
    let last_modified = file::last_modified(&source)?;

    if let Some(ast) = AST_CACHER.read().unwrap().get(&source, last_modified) {
        let meta = get_meta(source)?;
        return Ok((meta, ast.clone()));
    }

    let (meta, template) = match_meta_and_content(&source)?;
    AST_CACHER.write().unwrap().insert(&source, last_modified, template.clone());

    Ok((meta, template))
}

/// Get metadata from a source file, using sitemap cache if available
pub fn get_meta<P: AsRef<Path>>(source: P) -> CompileResult<AllayObject> {
    let map = SiteMap::read();
    let meta = map.urlset.get(&source.as_ref().to_path_buf());
    match meta {
        Some(entry) => Ok(entry.meta().as_ref().clone()),
        None => match_meta(source),
    }
}

/// Convert markdown text to HTML string using pulldown-cmark
pub fn convert_to_html(text: &str) -> String {
    let mut html_output = String::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_GFM);
    options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
    options.insert(Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS);

    let parser = Parser::new_ext(text, options);
    html::push_html(&mut html_output, parser);

    html_output
}
