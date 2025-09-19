use allay_base::data::{AllayData, AllayDataError, AllayObject};
use thiserror::Error;

use crate::ast::{Field, GetField};

#[derive(Debug, Error)]
pub enum InterpretError {
    #[error("{0}")]
    DataError(#[from] AllayDataError),

    #[error("Field not found: {0:?}")]
    FieldNotFound(Field),

    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(i32),
}

pub type InterpretResult<T> = Result<T, InterpretError>;

pub trait DataProvider {
    /// Get the data of the element
    fn get_data(&self) -> &AllayData;

    /// Get the field of the element
    fn get_field(&self, field: &Field) -> InterpretResult<&AllayData> {
        let mut cur = self.get_data();
        for f in &field.parts {
            match f {
                GetField::Index(i) => {
                    let list = cur.as_list()?;
                    cur = list.get(*i as usize).ok_or(InterpretError::IndexOutOfBounds(*i))?;
                }
                GetField::Name(name) => {
                    let obj = cur.as_obj()?;
                    cur = obj
                        .get(name)
                        .ok_or_else(|| InterpretError::FieldNotFound(field.clone()))?;
                }
            }
        }
        Ok(cur)
    }

    /// What the element renders to in template
    fn render(&self) -> String {
        self.get_data().to_string()
    }
}

/// The global site variable, usually from site config
#[derive(Debug, Clone)]
pub struct SiteVar {
    data: AllayData,
}

impl DataProvider for SiteVar {
    fn get_data(&self) -> &AllayData {
        &self.data
    }
}

/// The special variable `this`, which points to the current scope data
#[derive(Debug, Clone)]
pub struct ThisVar<'a> {
    scope: &'a TemplateScope<'a>,
}

impl<'a> DataProvider for ThisVar<'a> {
    fn get_data(&self) -> &AllayData {
        // HELP ME!
    }
}

/// A local variable defined in template, like `for $item: .items`
#[derive(Debug, Clone)]
pub struct LocalVar<'a> {
    pub id: String,
    pub data: &'a AllayData,
}

impl<'a> DataProvider for LocalVar<'a> {
    fn get_data(&self) -> &AllayData {
        &self.data
    }
}

/// An anonymous variable, used for interpreting expressions
#[derive(Debug, Clone)]
pub struct AnonymousVar<'a> {
    pub data: &'a AllayData,
}

impl<'a> DataProvider for AnonymousVar<'a> {
    fn get_data(&self) -> &AllayData {
        &self.data
    }
}

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
pub enum TemplateScope<'a> {
    Page(PageScope<'a>),
    Local(LocalScope<'a>),
}

/// The top level scope for a page, usually from the parent template or front-matter
#[derive(Debug, Clone)]
pub struct PageScope<'a> {
    pub inherited: &'a AllayObject,
    pub extra: AllayObject,
}

/// A local scope, usually created by `with` command
#[derive(Debug, Clone)]
pub struct LocalScope<'a> {
    pub parent: &'a TemplateScope<'a>,
    pub data: &'a AllayData,
}
