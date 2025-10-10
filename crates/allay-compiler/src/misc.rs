//! Miscellaneous utility functions for the Allay compiler.
//! These functions provide implementation for compiling source files (such as Markdown or HTML) into HTML strings.

use crate::env::{Compiled, Page};
use crate::interpret::Interpreter;
use crate::{CompileError, CompileResult, Compiler};
use crate::{CompileOutput, magic};
use allay_base::config::{get_allay_config, get_theme_path};
use allay_base::file;
use allay_base::template::ContentKind;
use std::path::{Path, PathBuf};

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
        kind: ContentKind,
    ) -> CompileResult<CompileOutput> {
        match kind {
            ContentKind::Article => self.article(source),
            ContentKind::General => self.general(source),
            ContentKind::Static => Err(CompileError::FileTypeNotSupported(
                source
                    .as_ref()
                    .to_path_buf()
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
            )),
        }
    }

    /// Mark an article source file as modified, so that all cached pages depending on it will be cleared.
    ///
    /// # Arguments
    /// - `source`: The path to the source file (markdown or html)
    /// - `kind`: The kind of content
    pub fn modify_file<P: AsRef<Path>>(
        &mut self,
        source: P,
        kind: ContentKind,
    ) -> CompileResult<()> {
        match kind {
            ContentKind::Article => self.modify_article(source),
            ContentKind::General => self.modify(source),
            ContentKind::Static => {}
        }
        Ok(())
    }

    /// Remove a source file from the cache and influenced mapping.
    ///
    /// # Arguments
    /// - `source`: The path to the source file (markdown or html)
    /// - `kind`: The kind of content
    pub fn remove_file<P: AsRef<Path>>(
        &mut self,
        source: P,
        kind: ContentKind,
    ) -> CompileResult<()> {
        match kind {
            ContentKind::Article => self.remove_article(source),
            ContentKind::General => self.remove(source),
            ContentKind::Static => {}
        }
        Ok(())
    }

    /// Compile a general file
    fn general<P: AsRef<Path>>(&mut self, source: P) -> CompileResult<CompileOutput> {
        let key = Self::default_key(&source);
        let source = source.as_ref().to_path_buf();

        let interpreter = &mut Self::default_interpreter();
        if let Some(page) = self.cache(&key) {
            // cached
            return page.compile(interpreter);
        }

        let page = Page::new(source.clone()).into();

        self.published.insert(key.clone());
        self.add(source.clone(), key.clone());
        self.remember(key, page.clone());

        page.compile(interpreter)
    }

    /// Get the template path for an article
    fn get_article_template<P: AsRef<Path>>(_article: P) -> PathBuf {
        // TODO: Support custom templates for articles (currently use the default "page.html")
        file::workspace(
            get_theme_path()
                .join(&get_allay_config().theme.template.dir)
                .join(&get_allay_config().theme.template.content),
        )
    }

    /// Generate a unique cache key for an article with its template
    fn template_article_key<P: AsRef<Path>>(template: P, article: P) -> String {
        format!(
            "{}|{}",
            Self::default_key(template),
            Self::default_key(article)
        )
    }

    /// Compile an article
    fn article<P: AsRef<Path>>(&mut self, article: P) -> CompileResult<CompileOutput> {
        let article = article.as_ref().into();
        let template = Self::get_article_template(&article);
        let article_key = Self::default_key(&article);
        let template_article_key = Self::template_article_key(&template, &article);

        let interpreter = &mut Self::default_interpreter();
        if let Some(page) = self.cache(&template_article_key) {
            // cached
            return page.compile(interpreter);
        }

        // generate an intermediate page based on its content (`sub` here)
        // listening to the article's changes with cache key `foo.md` (`article_key` here)
        // this page is just a <p>...</p> wrapper of the article content
        let sub = Page::new(article.clone()).into();
        self.add(article.clone(), article_key.clone());
        self.remember(article_key, sub.clone());

        // then generate the final page based on the template (`page` here)
        // listening to template's changes
        // Note that the template may generate many articles
        // so for each article, give a unique cache key, like `template|foo.md` (`template_article_key` here)
        let mut page = Page::new(template.clone());
        // replace the "content" key with the article page
        page.add_stash(magic::CONTENT.into(), sub);
        let page = page.into();

        self.published.insert(template_article_key.clone());
        self.add(template.clone(), template_article_key.clone());
        self.remember(template_article_key, page.clone());
        page.compile(interpreter)
    }

    fn modify_article<P: AsRef<Path>>(&mut self, article: P) {
        let template = Self::get_article_template(&article);

        self.modify(&article);
        self.modify(&template);
    }

    /// Remove an article and its associated template page from the cache and influenced mapping.
    fn remove_article<P: AsRef<Path>>(&mut self, article: P) {
        let template = Self::get_article_template(&article);

        self.remove(&article);
        self.remove(&template);
    }
}
