use crate::ast::Template;
use crate::interpret::interpret_meta;
use crate::parse::parse_file;
use crate::{CompileError, CompileResult, magic};
use allay_base::config::get_allay_config;
use allay_base::{data::AllayObject, file, template::TemplateKind};
use std::path::Path;
use std::sync::Arc;

fn post_preprocess<P: AsRef<Path>>(source: P, meta: &mut AllayObject) {
    meta.entry(magic::URL.into()).or_insert_with(|| {
        // Add the `url` field to the metadata
        let entry = source
            .as_ref()
            .strip_prefix(file::workspace(&get_allay_config().content.dir))
            .unwrap_or(source.as_ref());
        // for "foo\\bar.html", we change it to "/foo/bar.html"
        let url = Path::new("/")
            .join(entry)
            .with_extension(TemplateKind::Html.extension())
            .to_string_lossy()
            .to_string()
            .replace('\\', "/") // for Windows paths
            .into();
        Arc::new(url)
    });
}

pub fn get_meta_and_content<P: AsRef<Path>>(source: P) -> CompileResult<(AllayObject, Template)> {
    let kind = TemplateKind::from_filename(&source);
    let content = match kind {
        TemplateKind::Html | TemplateKind::Markdown => file::read_file_string(&source)?,
        TemplateKind::Other(e) => return Err(CompileError::FileTypeNotSupported(e)),
    };
    let ast = parse_file(&content)?;
    let mut meta = interpret_meta(&ast.meta)?;
    post_preprocess(source, &mut meta);

    Ok((meta, ast.template))
}

pub fn get_meta<P: AsRef<Path>>(source: P) -> CompileResult<AllayObject> {
    get_meta_and_content(source).map(|(m, _)| m)
}
