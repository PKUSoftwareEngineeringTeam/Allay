//! Miscellaneous utility functions for the Allay compiler.
//! These functions provide implementation for compiling source files (such as Markdown or HTML) into HTML strings.

use crate::env::{Compiled, Page};
use crate::extract::{convert_to_html, get_meta, match_raw_content};
use crate::interpret::Interpreter;
use crate::{CompileOutput, CompileResult, Compiler, magic};
use allay_base::config::{get_theme_config, get_theme_path};
use allay_base::file;
use allay_base::template::FileKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

impl Compiler<String> {
    /// the key generation method for caching
    fn default_key<P: AsRef<Path>>(path: P) -> String {
        path.as_ref().to_string_lossy().into()
    }

    /// A method to compile a raw source file into HTML string.
    /// This method does not use any caching mechanism.
    ///
    /// # Arguments
    /// - `source`: The path to the source file (markdown or html)
    /// - `include_dir`: The directory to look for included templates
    /// - `shortcode_dir`: The directory to look for shortcodes
    pub fn raw<P: AsRef<Path>>(
        source: P,
        include_dir: P,
        shortcode_dir: P,
    ) -> CompileResult<String> {
        let mut interpreter =
            Interpreter::new(include_dir.as_ref().into(), shortcode_dir.as_ref().into());
        let page = Page::new(source.as_ref().into());
        page.into().compile(&mut interpreter).map(|o| o.html)
    }

    /// Compile a source file with caching mechanism.
    ///
    /// # Arguments
    /// - `source`: The path to the source file (markdown or html)
    /// - `kind`: The kind of content
    pub fn compile_file<P: AsRef<Path>>(
        &mut self,
        source: P,
        kind: &FileKind,
    ) -> CompileResult<CompileOutput> {
        match kind {
            FileKind::Article => self.article(source),
            FileKind::Custom => self.custom(source),
            _ => unreachable!("Only article and general can be the compile entry"),
        }
    }

    /// Compile a general file
    fn custom(&mut self, source: impl AsRef<Path>) -> CompileResult<CompileOutput> {
        let key = Self::default_key(&source);
        let source = source.as_ref().to_path_buf();

        let interpreter = &mut Self::default_interpreter();
        if let Some(page) = self.cache(&key) {
            // cached
            return page.compile(interpreter);
        }

        let page = Page::new(source.clone()).into();

        self.publish(source, key.clone());
        self.remember(key, page.clone());

        page.compile(interpreter)
    }

    /// Get the wrapper path for an article
    fn get_article_wrapper(article: impl AsRef<Path>) -> CompileResult<PathBuf> {
        let meta = get_meta(article)?;

        let default = &get_theme_config().config.templates.content;
        let wrapper =
            meta.get(magic::TEMPLATE).and_then(|data| data.as_str().ok()).unwrap_or(default);

        let path = file::workspace(
            get_theme_path().join(&get_theme_config().config.templates.dir).join(wrapper),
        );

        Ok(path)
    }

    /// Generate a unique cache key for an article with its wrapper
    fn wrapper_article_key(wrapper: impl AsRef<Path>, article: impl AsRef<Path>) -> String {
        format!(
            "{}|{}",
            Self::default_key(wrapper),
            Self::default_key(article)
        )
    }

    /// Compile an article
    fn article(&mut self, article: impl AsRef<Path>) -> CompileResult<CompileOutput> {
        let wrapper = Self::get_article_wrapper(&article)?;

        let mut page = Page::new(wrapper.clone());
        let front_matter = get_meta(&article)?;

        // replace the "content" key with the article page
        let content = if front_matter
            .get(magic::RAW)
            .is_some_and(|value| value.as_bool().unwrap_or(false))
        {
            // raw content, do not compile the markdown
            convert_to_html(&match_raw_content(&article)?)
        } else {
            let key = Self::default_key(&article);
            let article_page =
                self.cache(&key).unwrap_or_else(|| Page::new(article.as_ref().into()).into());
            // the article page can also be cached
            // however, the actual page published is the wrapper page, so do not use `publish` here
            self.remember(key.clone(), article_page.clone());
            self.listen(&article, key);
            article_page.compile(&mut Self::default_interpreter())?.html
        };
        page.scope_mut().add_key(magic::CONTENT.into(), Arc::new(content.into()));
        // let the front matter of the article accessible in the wrapper
        page.scope_mut().merge_data(front_matter);

        let page = page.into();

        // note that the wrapper may generate many articles
        // so for each article, give a unique cache key, like `wrapper|foo.md`
        let key = Self::wrapper_article_key(&wrapper, &article);
        self.publish(&article, key.clone());
        // if the wrapper changes, the article also needs recompilation
        self.listen(wrapper.clone(), key.clone());
        self.remember(key, page.clone());

        page.compile(&mut Self::default_interpreter())
    }
}
