use crate::config::get_theme_config;
use crate::template::TemplateKind;
use std::path::{Path, PathBuf};

/// Represents a standardized URL path in Allay.
/// The path is normalized to handle common cases such as directory paths and HTML files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllayUrlPath {
    /// a directory path ending with '/'
    Index(PathBuf),
    /// an HTML file without extension
    Html(PathBuf),
    /// other file types
    Other(PathBuf),
}

impl AsRef<Path> for AllayUrlPath {
    fn as_ref(&self) -> &Path {
        match self {
            AllayUrlPath::Index(p) => p.as_ref(),
            AllayUrlPath::Html(p) => p.as_ref(),
            AllayUrlPath::Other(p) => p.as_ref(),
        }
    }
}

impl AllayUrlPath {
    /// Create a standard [`AllayUrlPath`] from a given path.
    /// The path is normalized according to the following rules:
    ///
    /// 1. If the path is an HTML file and is named `index.html`, it is converted to a directory path.
    /// 2. If the path is an HTML file but not named `index.html`, the extension is removed.
    /// 3. If the path is not an HTML file, it is returned as is.
    ///
    /// # Examples
    ///
    /// ```
    /// use allay_base::url::AllayUrlPath;
    /// use std::path::Path;
    ///
    /// // Case 1: index.html file
    /// assert_eq!(AllayUrlPath::from("blog/index.html").as_ref(), Path::new("blog/"));
    /// // Case 2: other HTML file
    /// assert_eq!(AllayUrlPath::from("about.html").as_ref(), Path::new("about"));
    /// // Case 3: non-HTML file
    /// assert_eq!(AllayUrlPath::from("styles/main.css").as_ref(), Path::new("styles/main.css"));
    /// ```
    pub fn from(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        if Self::is_dir(path) {
            AllayUrlPath::Index(path.into())
        } else if let Some(ext) = path.extension()
            && let Some(ext) = ext.to_str()
            && TemplateKind::from_extension(ext).is_html()
        {
            if path.file_name() == Some(get_theme_config().config.templates.index.as_ref()) {
                AllayUrlPath::Index(path.parent().map(Self::to_dir).unwrap_or_default())
            } else {
                AllayUrlPath::Html(path.with_extension(""))
            }
        } else {
            AllayUrlPath::Other(path.into())
        }
    }

    /// Get all possible file paths for this URL path.
    /// This includes:
    /// 1. If the path is not a directory, the path itself.
    /// 2. If the path is a directory, the `index.html` file inside it.
    /// 3. the `.html` file with the same name.
    ///
    /// # Examples
    ///
    /// ```
    /// use allay_base::url::AllayUrlPath;
    /// use std::path::PathBuf;
    ///
    /// assert_eq!(
    ///     AllayUrlPath::from("blog/").possible_paths(),
    ///     vec![PathBuf::from("blog/index.html")]
    /// );
    /// assert_eq!(
    ///     AllayUrlPath::from("about").possible_paths(),
    ///     vec![PathBuf::from("about"), PathBuf::from("about.html")]
    /// );
    /// assert_eq!(
    ///     AllayUrlPath::from("styles/main.css").possible_paths(),
    ///     vec![PathBuf::from("styles/main.css"), PathBuf::from("styles/main.css.html")]
    /// );
    /// ```
    pub fn possible_paths(&self) -> Vec<PathBuf> {
        match self {
            AllayUrlPath::Index(p) => vec![p.join(&get_theme_config().config.templates.index)],
            AllayUrlPath::Html(p) => {
                vec![p.clone(), p.with_extension(TemplateKind::Html.extension())]
            }
            AllayUrlPath::Other(p) => {
                vec![
                    p.clone(),
                    p.with_added_extension(TemplateKind::Html.extension()),
                ]
            }
        }
    }

    pub fn is_dir(path: impl AsRef<Path>) -> bool {
        path.as_ref().to_string_lossy().ends_with("/")
    }

    pub fn to_dir(path: impl AsRef<Path>) -> PathBuf {
        let str = path.as_ref().to_string_lossy();
        if str.ends_with("/") {
            PathBuf::from(str.as_ref())
        } else {
            PathBuf::from(format!("{}/", str))
        }
    }
}
