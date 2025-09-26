#![allow(dead_code)] // TODO: remove this line when the module is complete

use crate::InterpretResult;
use crate::ast::GetField;
use crate::interpret::traits::{DataProvider, Variable};
use allay_base::data::{AllayData, AllayList};
use std::sync::{Arc, OnceLock};

/// The global site variable, usually from site config
#[derive(Debug, Clone)]
pub(crate) struct SiteVar {
    pub data: Arc<AllayData>,
}

impl DataProvider for SiteVar {
    fn get_data(&self) -> Arc<AllayData> {
        self.data.clone()
    }
}

impl Variable for SiteVar {}

impl SiteVar {
    pub fn get_instance() -> &'static SiteVar {
        static SITE_INSTANCE: OnceLock<SiteVar> = OnceLock::new();
        SITE_INSTANCE.get_or_init(|| {
            let site_config = allay_base::config::site::get_site_config();
            let site_data = Arc::new(AllayData::from(site_config.clone()));
            SiteVar { data: site_data }
        })
    }
}

/// The special variable `this`, which points to the current scope data
#[derive(Clone)]
pub(crate) struct ThisVar<'a> {
    provider: &'a dyn DataProvider,
}

impl<'a> ThisVar<'a> {
    pub fn create(scope: &'a dyn DataProvider) -> Self {
        ThisVar { provider: scope }
    }
}

impl DataProvider for ThisVar<'_> {
    fn get_data(&self) -> Arc<AllayData> {
        self.provider.get_data()
    }

    fn get_field(&self, fields: &[GetField]) -> InterpretResult<Arc<AllayData>> {
        self.provider.get_field(fields)
    }
}

impl Variable for ThisVar<'_> {}

/// The special variable `param`, which is often set by parents.
/// It is actually an [`AllayList`]` of different parameters
#[derive(Clone, Debug, Default)]
pub(crate) struct ParamVar {
    data: Arc<AllayData>,
}

impl ParamVar {
    pub fn create(data: AllayList) -> Self {
        ParamVar {
            data: Arc::new(AllayData::from(data)),
        }
    }
}

impl DataProvider for ParamVar {
    fn get_data(&self) -> Arc<AllayData> {
        self.data.clone()
    }
}

impl Variable for ParamVar {}

/// A local variable defined in template, like `for $item: .items`
/// or the implicit(anonymous) variables
#[derive(Debug, Clone)]
pub(crate) struct LocalVar {
    data: Arc<AllayData>,
}

impl LocalVar {
    pub fn create(data: Arc<AllayData>) -> Self {
        LocalVar { data }
    }
}

impl DataProvider for LocalVar {
    fn get_data(&self) -> Arc<AllayData> {
        self.data.clone()
    }
}

impl Variable for LocalVar {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpret::{
        scope::{LocalScope, PageScope},
        traits::Scope,
    };
    use allay_base::data::{AllayList, AllayObject};
    use std::sync::{Arc, LazyLock};

    // inherited: {"author": "Alice", "date": "2023-10-01"}
    static PARENT: LazyLock<Arc<AllayObject>> = LazyLock::new(|| {
        Arc::new(AllayObject::from([
            ("author".into(), Arc::new(AllayData::from("Alice"))),
            ("date".into(), Arc::new(AllayData::from("2023-10-01"))),
        ]))
    });

    fn gen_page_scope() -> PageScope {
        // params: ["param1", 42]
        let params = AllayList::from([
            Arc::new(AllayData::from("param1")),
            Arc::new(AllayData::from(42)),
        ]);

        let mut page = PageScope::new_from(PARENT.clone(), params);
        // owned: {"title": "My Page", "tags": ["test", "markdown"]}
        page.add_key("title".into(), AllayData::from("My Page"));
        page.add_key(
            "tags".into(),
            AllayData::from(AllayList::from([
                Arc::new(AllayData::from("test")),
                Arc::new(AllayData::from("markdown")),
            ])),
        );
        page
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
        let local = LocalScope::new(LocalVar::create(
            this.get_field(&[GetField::Name("tags".into())]).unwrap(),
        ));
        assert_eq!(local.create_this().get_data().as_list().unwrap().len(), 2);
    }
}
