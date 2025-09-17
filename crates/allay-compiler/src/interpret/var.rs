#![allow(dead_code)] // TODO: remove this line when the module is complete

use crate::ast::GetField;
use crate::interpret::scope::{PageScope, Scope};
use crate::{InterpretError, InterpretResult};
use allay_base::data::{AllayData, AllayDataError, AllayObject};
use std::cell::OnceCell;

pub(crate) trait Variable {
    /// Get the data of the element
    fn get_data(&self) -> &AllayData;

    /// Utility function to get the field of the element once
    fn get_field_once<'a>(cur: &'a AllayData, field: &GetField) -> InterpretResult<&'a AllayData> {
        match field {
            GetField::Index(i) => {
                let list = cur.as_list()?;
                list.get(*i).ok_or(InterpretError::IndexOutOfBounds(*i))
            }
            GetField::Name(name) => {
                let obj = cur.as_obj()?;
                obj.get(name).ok_or(InterpretError::FieldNotFound(name.clone()))
            }
        }
    }

    /// Get the field of the element
    fn get_field(&self, fields: &Vec<GetField>) -> InterpretResult<&AllayData> {
        fields.iter().try_fold(self.get_data(), Self::get_field_once)
    }

    /// What the element renders to in template
    fn render(&self) -> String {
        self.get_data().to_string()
    }
}

/// The global site variable, usually from site config
/// TODO: Implement it as a singleton
#[derive(Debug, Clone)]
pub(crate) struct SiteVar {
    pub data: AllayData,
}

impl Variable for SiteVar {
    fn get_data(&self) -> &AllayData {
        &self.data
    }
}

/// The special variable `this`, which points to the current scope data
#[derive(Debug, Clone)]
pub(crate) struct ThisVar<'a> {
    pub scope: &'a Scope<'a>,
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

    fn get_field(&self, fields: &Vec<GetField>) -> InterpretResult<&AllayData> {
        // Optimized implementation without using get_data()
        match self.scope {
            Scope::Local(local) => fields.iter().try_fold(local.data, Self::get_field_once),
            Scope::Page(page) => {
                let first =
                    fields.first().ok_or(InterpretError::FieldNotFound("Empty field".into()))?;

                if let GetField::Name(name) = first {
                    let cur = if page.owned.contains_key(name) {
                        page.owned.get(name).unwrap()
                    } else if let Some(inherited) = page.inherited {
                        inherited.get(name).ok_or(InterpretError::FieldNotFound(name.clone()))?
                    } else {
                        return Err(InterpretError::FieldNotFound(name.clone()));
                    };

                    fields[1..].iter().try_fold(cur, Self::get_field_once)
                } else {
                    // Page scope is always an object
                    Err(InterpretError::DataError(AllayDataError::TypeConversion(
                        "Page scope is not a list".to_string(),
                    )))
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
