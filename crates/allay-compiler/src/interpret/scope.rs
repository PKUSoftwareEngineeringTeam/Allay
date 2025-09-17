#![allow(dead_code)] // TODO: remove this line when the module is complete

use allay_base::data::{AllayData, AllayList, AllayObject};

/// The variable scope for template, organized as a tree like json object
///
/// # Example
/// Current scope:
/// ```json
/// {
///  "title": "Hello, world!",
///  "author": {
///    "name": "John Doe",
///    "age": 30
///  },
///  "tags": ["test", "markdown"]
/// }
/// ```
///
/// Then the template can be like:
/// ```html
/// <!-- visit variables by dot notation -->
/// <h1>{: .title :}</h1>
///
/// <!-- use "for" to iterate a list -->
/// {- for $tag: .tags -}
/// <span>{: $tag :}</span>
/// {- end -}
///
/// <!-- use "with" to visit a child scope -->
/// {- with .author -}
/// <p>Author: {: .name :}, Age: {: .age :}</p>
/// {- end -}
/// ```
#[derive(Debug, Clone)]
pub(crate) enum Scope<'a> {
    Page(PageScope<'a>),
    Local(LocalScope<'a>),
}

/// The top level scope for a page, usually from the parent template or front-matter
///
/// Note: Owned data has higher priority, which means if both inherited and owned have the same key,
/// the value in extra will be used.
#[derive(Debug, Clone)]
pub(crate) struct PageScope<'a> {
    pub owned: AllayObject,
    pub inherited: Option<&'a AllayObject>,
    pub params: AllayList,
}

impl PageScope<'_> {
    /// The scope of top level pages with no inherited data.
    /// Usually for the markdown contents
    /// or the magic pages like "index.html" or "404.html"
    pub fn new_top(data: AllayObject, params: AllayList) -> PageScope<'static> {
        PageScope {
            owned: data,
            inherited: None,
            params,
        }
    }

    pub fn new(owned: AllayObject, inherited: &AllayObject, params: AllayList) -> PageScope<'_> {
        PageScope {
            owned,
            inherited: Some(inherited),
            params,
        }
    }
}

/// A local scope, usually created by `with` command
#[derive(Debug, Clone)]
pub(crate) struct LocalScope<'a> {
    pub parent: &'a Scope<'a>,
    pub data: &'a AllayData,
}
