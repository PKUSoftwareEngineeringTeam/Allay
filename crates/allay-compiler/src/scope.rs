#![allow(dead_code)] // TODO: remove this line when the module is complete
// TODO: May reconstruct the module structure later

use crate::ast::{Field, GetField};
use crate::{InterpretError, InterpretResult};
use allay_base::data::{AllayData, AllayDataError, AllayList, AllayObject};
use std::cell::OnceCell;

pub(crate) trait Variable {
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

/// The global site variable, usually from site config
/// TODO: Implement it as a singleton
#[derive(Debug, Clone)]
pub(crate) struct SiteVar {
    data: AllayData,
}

impl Variable for SiteVar {
    fn get_data(&self) -> &AllayData {
        &self.data
    }
}

/// The special variable `this`, which points to the current scope data
#[derive(Debug, Clone)]
pub(crate) struct ThisVar<'a> {
    scope: &'a Scope<'a>,
    /// the merged data, cached for performance
    /// it is hardly used, except for `{: this :}` expression
    merged: OnceCell<AllayData>,
}

impl Variable for ThisVar<'_> {
    fn get_data(&self) -> &AllayData {
        match self.scope {
            Scope::Local(local) => local.data,
            Scope::Page(page) => {
                self.merged.get_or_init(|| {
                    // Merge inherited and extra data
                    let mut merged = page.inherited.unwrap_or(&AllayObject::new()).clone();

                    for (k, v) in page.owned.clone() {
                        merged.insert(k, v);
                    }
                    AllayData::from(merged)
                })
            }
        }
    }

    fn get_field(&self, field: &Field) -> InterpretResult<&AllayData> {
        // Optimized implementation without using get_data()
        match self.scope {
            Scope::Local(local) => {
                let mut cur = local.data;
                for f in &field.parts {
                    cur = Self::get_field_once(cur, f)?;
                }
                Ok(cur)
            }
            Scope::Page(page) => {
                let first = field
                    .parts
                    .first()
                    .ok_or(InterpretError::FieldNotFound("Empty field".into()))?;

                match first {
                    GetField::Index(_) => {
                        // Page scope is always an object
                        Err(InterpretError::DataError(AllayDataError::TypeConversion(
                            "Page scope is not a list".to_string(),
                        )))
                    }
                    GetField::Name(name) => {
                        let mut cur = if page.owned.contains_key(name) {
                            page.owned.get(name).unwrap()
                        } else if let Some(inherited) = page.inherited {
                            inherited
                                .get(name)
                                .ok_or(InterpretError::FieldNotFound(name.clone()))?
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
    }
}

/// The special variable `param`, which is often set by parents
#[derive(Debug, Clone)]
pub(crate) struct ParamVar<'a> {
    pub scope: &'a PageScope<'a>,
    pub index: usize,
}

impl Variable for ParamVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.scope.params.get(self.index).unwrap_or(&AllayData::Null)
    }
}

/// A local variable defined in template, like `for $item: .items`
#[derive(Debug, Clone)]
pub(crate) struct LocalVar<'a> {
    pub id: String,
    pub data: &'a AllayData,
}

impl Variable for LocalVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.data
    }
}

/// An anonymous variable, used for interpreting expressions
#[derive(Debug, Clone)]
pub(crate) struct AnonymousVar<'a> {
    pub data: &'a AllayData,
}

impl Variable for AnonymousVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.data
    }
}
