use crate::ast::GetField;
use crate::interpret::traits::{DataProvider, Scope, get_field_once};
use crate::interpret::var::{LocalVar, ParamVar, ThisVar};
use crate::{InterpretError, InterpretResult};
use allay_base::data::{AllayData, AllayDataError, AllayList, AllayObject};
use std::cell::OnceCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

/// The top level scope for a page, usually from the parent template or front-matter
///
/// Note: Owned data has higher priority, which means if both inherited and owned have the same key,
/// the value in extra will be used.
#[derive(Clone, Debug, Default)]
pub struct PageScope {
    owned: Arc<AllayObject>,
    inherited: Option<Arc<AllayObject>>,

    sub_stack: Vec<LocalScope>,
    locals: HashMap<String, LocalVar>,
    param: ParamVar,

    /// the merged data of `this`, cached for performance.
    /// It is hardly used, except for `{: this :}` expression in page scope
    merged: OnceCell<Arc<AllayData>>,
}

impl PageScope {
    pub fn new() -> PageScope {
        PageScope::default()
    }

    pub fn new_from(inherited: Arc<AllayObject>, params: AllayList) -> PageScope {
        let mut page = PageScope::new();
        page.param = ParamVar::create(params);
        page.inherited = Some(inherited);
        page
    }

    pub fn add_key(&mut self, key: String, value: Arc<AllayData>) {
        Arc::make_mut(&mut self.owned).insert(key, value);
        self.merged.take();
    }

    pub fn create_sub_scope(&mut self, var: LocalVar) -> &mut LocalScope {
        self.sub_stack.push(LocalScope::new(var));
        self.sub_stack.last_mut().unwrap()
    }

    pub fn exit_sub_scope(&mut self) -> Option<LocalScope> {
        self.sub_stack.pop()
    }

    pub fn cur_scope(&self) -> &dyn Scope {
        if self.sub_stack.is_empty() {
            self
        } else {
            self.sub_stack.last().unwrap()
        }
    }

    pub fn cur_scope_mut(&mut self) -> &mut dyn Scope {
        if self.sub_stack.is_empty() {
            self
        } else {
            self.sub_stack.last_mut().unwrap()
        }
    }

    pub fn get_local(&self, id: &str) -> Option<&LocalVar> {
        // find the variable in local scopes first
        for scope in self.sub_stack.iter().rev() {
            if let Some(var) = scope.locals.get(id) {
                return Some(var);
            }
        }
        self.locals.get(id)
    }

    pub fn get_param(&self) -> &ParamVar {
        &self.param
    }
}

impl DataProvider for PageScope {
    /// Warning: Directly access `this` in page scope is not recommended,
    /// because it requires merging the data, which is not efficient.
    /// Access field directly by [`Self::get_field`] instead as much as possible.
    fn get_data(&self) -> Arc<AllayData> {
        self.merged
            .get_or_init(|| {
                // Merge inherited and extra data
                let mut merged = if let Some(inherited) = &self.inherited {
                    inherited.deref().clone()
                } else {
                    AllayObject::new()
                };

                for (k, v) in self.owned.deref().clone() {
                    merged.insert(k, v);
                }
                Arc::new(AllayData::from(merged))
            })
            .clone()
    }

    fn get_field(&self, fields: &[GetField]) -> InterpretResult<Arc<AllayData>> {
        // Optimized implementation without using get_data()
        let first = fields.first().ok_or(InterpretError::FieldNotFound("Empty field".into()))?;

        if let GetField::Name(name) = first {
            let cur = if self.owned.contains_key(name) {
                // clone an Arc is ok here
                self.owned.get(name).unwrap().clone()
            } else if let Some(inherited) = &self.inherited {
                inherited.get(name).unwrap_or(&Arc::new(AllayData::Null)).clone()
            } else {
                return Ok(Arc::new(AllayData::Null));
            };

            fields[1..].iter().try_fold(cur.clone(), get_field_once)
        } else {
            // Page scope is always an object
            Err(InterpretError::DataError(AllayDataError::TypeConversion(
                "Page scope is not a list".into(),
            )))
        }
    }
}

impl Scope for PageScope {
    fn create_this(&self) -> ThisVar<'_> {
        ThisVar::create(self)
    }

    fn create_local_var(&mut self, id: String, data: LocalVar) {
        self.locals.insert(id, data);
    }
}

/// A local scope, usually created by `with` command
#[derive(Clone, Debug)]
pub(crate) struct LocalScope {
    this: LocalVar,
    pub(super) locals: HashMap<String, LocalVar>,
}

impl LocalScope {
    pub fn new(var: LocalVar) -> LocalScope {
        LocalScope {
            this: var,
            locals: HashMap::new(),
        }
    }
}

impl DataProvider for LocalScope {
    fn get_data(&self) -> Arc<AllayData> {
        self.this.get_data()
    }
}

impl Scope for LocalScope {
    fn create_this(&self) -> ThisVar<'_> {
        ThisVar::create(self)
    }

    fn create_local_var(&mut self, id: String, data: LocalVar) {
        self.locals.insert(id, data);
    }
}
