#![allow(dead_code)] // TODO: remove this line when the module is complete
// TODO: May reconstruct the module structure later

use crate::ast::{Field, GetField};
use crate::error::{InterpretError, InterpretResult};
use allay_base::data::{AllayData, AllayDataError, AllayObject};
use std::cell::OnceCell;

pub(crate) trait DataProvider {
    /// Get the data of the element
    fn get_data(&self) -> &AllayData;

    /// Utility function to get the field of the element once
    fn get_field_once<'a>(cur: &'a AllayData, layer: &GetField) -> InterpretResult<&'a AllayData> {
        match layer {
            GetField::Index(i) => {
                if (*i) < 0 {
                    return Err(InterpretError::IndexOutOfBounds(*i));
                }
                let list = cur.as_list()?;
                list.get(*i as usize).ok_or(InterpretError::IndexOutOfBounds(*i))
            }
            GetField::Name(name) => {
                let obj = cur.as_obj()?;
                obj.get(name).ok_or(InterpretError::FieldNotFound(name.clone()))
            }
        }
    }

    /// Get the field of the element
    fn get_field(&self, field: &Field) -> InterpretResult<&AllayData> {
        let mut cur = self.get_data();
        for f in &field.parts {
            cur = Self::get_field_once(cur, f)?;
        }
        Ok(cur)
    }

    /// What the element renders to in template
    fn render(&self) -> String {
        self.get_data().to_string()
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
pub(crate) enum TemplateScope<'a> {
    Page(PageScope<'a>),
    Local(LocalScope<'a>),
}

impl DataProvider for TemplateScope<'_> {
    fn get_data(&self) -> &AllayData {
        match self {
            TemplateScope::Page(page) => page.get_data(),
            TemplateScope::Local(local) => local.get_data(),
        }
    }
}

/// The top level scope for a page, usually from the parent template or front-matter
///
/// Note: Owned data has higher priority, which means if both inherited and owned have the same key,
/// the value in extra will be used.
#[derive(Debug, Clone)]
pub(crate) struct PageScope<'a> {
    pub owned: AllayObject,
    pub inherited: Option<&'a AllayObject>,
    /// the merged data, cached for performance
    /// it is hardly used, except for `{: this :}` expression
    merged: OnceCell<AllayData>,
}

impl PageScope<'_> {
    /// The scope of top level pages with no inherited data.
    /// Usually for the markdown contents
    /// or the magic pages like "index.html" or "404.html"
    pub fn new_top(data: AllayObject) -> PageScope<'static> {
        PageScope {
            owned: data,
            inherited: None,
            merged: OnceCell::new(),
        }
    }

    pub fn new(owned: AllayObject, inherited: &AllayObject) -> PageScope<'_> {
        PageScope {
            owned,
            inherited: Some(inherited),
            merged: OnceCell::new(),
        }
    }
}

impl DataProvider for PageScope<'_> {
    fn get_data(&self) -> &AllayData {
        self.merged.get_or_init(|| {
            // Merge inherited and extra data
            let mut merged = self.inherited.unwrap_or(&AllayObject::new()).clone();

            for (k, v) in self.owned.clone() {
                merged.insert(k, v);
            }
            AllayData::from(merged)
        })
    }

    fn get_field(&self, field: &Field) -> InterpretResult<&AllayData> {
        // Optimized implementation without using get_data()
        let first =
            field.parts.first().ok_or(InterpretError::FieldNotFound("Empty field".into()))?;

        match first {
            GetField::Index(_) => {
                // Page scope is always an object
                Err(InterpretError::DataError(AllayDataError::TypeConversion(
                    "Page scope is not a list".to_string(),
                )))
            }
            GetField::Name(name) => {
                let mut cur = if self.owned.contains_key(name) {
                    self.owned.get(name).unwrap()
                } else if let Some(inherited) = self.inherited {
                    inherited.get(name).ok_or(InterpretError::FieldNotFound(name.clone()))?
                } else {
                    return Err(InterpretError::FieldNotFound(name.clone()));
                };

                for layer in &field.parts[1..] {
                    cur = Self::get_field_once(cur, layer)?;
                }

                Ok(cur)
            }
        }
    }
}

/// A local scope, usually created by `with` command
#[derive(Debug, Clone)]
pub(crate) struct LocalScope<'a> {
    pub parent: &'a TemplateScope<'a>,
    pub data: &'a AllayData,
}

impl DataProvider for LocalScope<'_> {
    fn get_data(&self) -> &AllayData {
        self.data
    }
}

/// The global site variable, usually from site config
/// TODO: Implement it as a singleton
#[derive(Debug, Clone)]
pub(crate) struct SiteVar {
    data: AllayData,
}

impl DataProvider for SiteVar {
    fn get_data(&self) -> &AllayData {
        &self.data
    }
}

/// The special variable `this`, which points to the current scope data
#[derive(Debug, Clone)]
pub(crate) struct ThisVar<'a> {
    scope: &'a TemplateScope<'a>,
}

impl DataProvider for ThisVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.scope.get_data()
    }
}

/// A local variable defined in template, like `for $item: .items`
#[derive(Debug, Clone)]
pub(crate) struct LocalVar<'a> {
    pub id: String,
    pub data: &'a AllayData,
}

impl DataProvider for LocalVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.data
    }
}

/// An anonymous variable, used for interpreting expressions
#[derive(Debug, Clone)]
pub(crate) struct AnonymousVar<'a> {
    pub data: &'a AllayData,
}

impl DataProvider for AnonymousVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.data
    }
}
