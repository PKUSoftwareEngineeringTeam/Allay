#![allow(dead_code)] // TODO: remove this line when the module is complete

use crate::InterpretResult;
use crate::ast::GetField;
use crate::interpret::scope::PageScope;
use crate::interpret::traits::{DataProvider, Scope, Variable};
use allay_base::data::AllayData;

/// The global site variable, usually from site config
/// TODO: Implement it as a singleton
#[derive(Debug, Clone)]
pub(crate) struct SiteVar {
    pub data: AllayData,
}

impl DataProvider for SiteVar {
    fn get_data(&self) -> &AllayData {
        &self.data
    }
}

impl Variable for SiteVar {}

/// The special variable `this`, which points to the current scope data
#[derive(Clone)]
pub(crate) struct ThisVar<'a> {
    pub scope: &'a dyn Scope,
}

impl<'a> ThisVar<'a> {
    pub fn create(scope: &'a dyn Scope) -> ThisVar<'a> {
        ThisVar { scope }
    }
}

impl DataProvider for ThisVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.scope.get_data()
    }

    fn get_field(&self, fields: &[GetField]) -> InterpretResult<&AllayData> {
        self.scope.get_field(fields)
    }
}

impl Variable for ThisVar<'_> {}

/// The special variable `param`, which is often set by parents
#[derive(Clone, Debug)]
pub(crate) struct ParamVar<'a> {
    pub scope: &'a PageScope<'a>,
    pub index: usize,
}

impl<'a> ParamVar<'a> {
    pub fn create(scope: &'a PageScope, index: usize) -> ParamVar<'a> {
        ParamVar { scope, index }
    }
}

impl DataProvider for ParamVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.scope.params.get(self.index).unwrap_or(&AllayData::Null)
    }
}

impl Variable for ParamVar<'_> {}

/// A local variable defined in template, like `for $item: .items`
#[derive(Debug, Clone)]
pub(crate) struct LocalVar<'a> {
    pub id: String,
    pub data: &'a AllayData,
}

impl LocalVar<'_> {
    pub fn create(id: String, data: &AllayData) -> LocalVar<'_> {
        LocalVar { id, data }
    }
}

impl DataProvider for LocalVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.data
    }
}

impl Variable for LocalVar<'_> {}

/// An anonymous variable, used for interpreting expressions
#[derive(Debug, Clone)]
pub(crate) struct AnonymousVar<'a> {
    pub data: &'a AllayData,
}

impl AnonymousVar<'_> {
    pub fn create(data: &AllayData) -> AnonymousVar<'_> {
        AnonymousVar { data }
    }
}

impl DataProvider for AnonymousVar<'_> {
    fn get_data(&self) -> &AllayData {
        self.data
    }
}

impl Variable for AnonymousVar<'_> {}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use crate::interpret::scope::LocalScope;

    use super::*;
    use allay_base::data::{AllayList, AllayObject};

    static PARENT: LazyLock<AllayObject> = LazyLock::new(|| {
        AllayObject::from([
            ("author".into(), AllayData::from("Alice")),
            ("date".into(), AllayData::from("2023-10-01")),
        ])
    });

    fn gen_page_scope() -> PageScope<'static> {
        // owned: {"title": "My Page", "tags": ["test", "markdown"]}
        let owned = AllayObject::from([
            ("title".into(), AllayData::from("My Page")),
            (
                "tags".into(),
                AllayData::from(AllayList::from([
                    AllayData::from("test"),
                    AllayData::from("markdown"),
                ])),
            ),
        ]);
        // inherited: {"author": "Alice", "date": "2023-10-01"}
        // params: ["param1", 42]
        let params = AllayList::from([AllayData::from("param1"), AllayData::from(42)]);

        PageScope::new(owned, &PARENT, params)
    }

    #[test]
    fn test_vars() {
        let scope = gen_page_scope();
        let this_var = scope.create_this();

        // get_data
        let data = this_var.get_data();
        assert!(data.is_obj());

        // get_field
        let title = this_var.get_field(&[GetField::Name("title".into())]).unwrap();
        assert_eq!(title.as_str().unwrap(), "My Page");

        let author = this_var.get_field(&[GetField::Name("author".into())]).unwrap();
        assert_eq!(author.as_str().unwrap(), "Alice");

        let tag0 = this_var
            .get_field(&[GetField::Name("tags".into()), GetField::Index(0)])
            .unwrap();
        assert_eq!(tag0.as_str().unwrap(), "test");
    }

    #[test]
    fn test_local_scope() {
        let scope = gen_page_scope();
        let this = scope.create_this();
        let local = LocalScope::new(
            &scope,
            this.get_field(&[GetField::Name("tags".into())]).unwrap(),
        );
        assert_eq!(local.create_this().get_data().as_list().unwrap().len(), 2);
    }
}
