#![allow(dead_code)] // TODO: remove this line when the module is complete

use std::cell::OnceCell;

use crate::ast::GetField;
use crate::interpret::traits::{DataProvider, Scope, get_field_once};
use crate::{InterpretError, InterpretResult};
use allay_base::data::{AllayData, AllayDataError, AllayList, AllayObject};

/// The top level scope for a page, usually from the parent template or front-matter
///
/// Note: Owned data has higher priority, which means if both inherited and owned have the same key,
/// the value in extra will be used.
#[derive(Clone, Debug)]
pub(crate) struct PageScope<'a> {
    pub owned: AllayObject,
    pub inherited: Option<&'a AllayObject>,
    pub params: AllayList,

    /// the merged data of `this`, cached for performance
    /// it is hardly used, except for `{: this :}` expression in page scope
    merged: OnceCell<AllayData>,
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
            merged: OnceCell::new(),
        }
    }

    pub fn new(owned: AllayObject, inherited: &AllayObject, params: AllayList) -> PageScope<'_> {
        PageScope {
            owned,
            inherited: Some(inherited),
            params,
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

    fn get_field(&self, fields: &[GetField]) -> InterpretResult<&AllayData> {
        // Optimized implementation without using get_data()
        let first = fields.first().ok_or(InterpretError::FieldNotFound("Empty field".into()))?;

        if let GetField::Name(name) = first {
            let cur = if self.owned.contains_key(name) {
                self.owned.get(name).unwrap()
            } else if let Some(inherited) = self.inherited {
                inherited.get(name).ok_or(InterpretError::FieldNotFound(name.clone()))?
            } else {
                return Err(InterpretError::FieldNotFound(name.clone()));
            };

            fields[1..].iter().try_fold(cur, get_field_once)
        } else {
            // Page scope is always an object
            Err(InterpretError::DataError(AllayDataError::TypeConversion(
                "Page scope is not a list".into(),
            )))
        }
    }
}

impl Scope for PageScope<'_> {}

/// A local scope, usually created by `with` command
#[derive(Clone)]
pub(crate) struct LocalScope<'a> {
    pub parent: &'a dyn Scope,
    pub data: &'a AllayData,
}

impl<'a> LocalScope<'a> {
    pub fn new(parent: &'a dyn Scope, data: &'a AllayData) -> LocalScope<'a> {
        LocalScope { parent, data }
    }
}

impl DataProvider for LocalScope<'_> {
    fn get_data(&self) -> &AllayData {
        self.data
    }
}

impl Scope for LocalScope<'_> {}
