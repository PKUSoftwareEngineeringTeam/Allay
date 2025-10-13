use crate::InterpretResult;
use crate::ast::GetField;
use crate::interpret::traits::{DataProvider, Variable};
use crate::meta::get_meta;
use allay_base::config::{get_allay_config, get_site_config};
use allay_base::data::{AllayData, AllayList};
use allay_base::file;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// The global site variable, usually from site config
#[derive(Debug)]
pub struct SiteVar {
    pub data: Arc<AllayData>,
}

impl SiteVar {
    pub fn get_instance() -> &'static SiteVar {
        static INSTANCE: OnceLock<SiteVar> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let site_data = get_site_config().get("params").cloned().unwrap_or_default();
            SiteVar { data: site_data }
        })
    }
}

impl DataProvider for SiteVar {
    fn get_data(&self) -> Arc<AllayData> {
        self.data.clone()
    }
}

impl Variable for SiteVar {}

#[derive(Debug)]
pub struct PagesVar {
    data: RwLock<Arc<AllayData>>,
    locked: bool,
}

impl PagesVar {
    pub fn get_instance() -> &'static Mutex<PagesVar> {
        static INSTANCE: OnceLock<Mutex<PagesVar>> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let instance = PagesVar {
                data: RwLock::new(Arc::new(AllayList::new().into())),
                locked: false,
            };
            instance.update();
            Mutex::new(instance)
        })
    }

    pub fn update(&self) {
        let dir = file::workspace(&get_allay_config().content.dir);
        // walk through the content directory and get all markdown/html files
        if self.locked {
            return;
        }
        if let Ok(entries) = file::read_dir_all_files(&dir) {
            let data = entries
                .into_iter()
                .filter_map(|e| get_meta(e).ok())
                .map(AllayData::from)
                .map(Arc::new)
                .collect::<AllayList>()
                .into();
            *self.data.write().unwrap() = Arc::new(data);
        }
    }
}

impl DataProvider for Mutex<PagesVar> {
    fn get_data(&self) -> Arc<AllayData> {
        let lock = self.lock().unwrap();
        lock.data.read().unwrap().clone()
    }
}

impl Variable for Mutex<PagesVar> {}

/// The special variable `this`, which points to the current scope data
#[derive(Clone)]
pub struct ThisVar<'a> {
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
pub struct ParamVar {
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
pub struct LocalVar {
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
        page.add_key("title".into(), Arc::new(AllayData::from("My Page")));
        page.add_key(
            "tags".into(),
            Arc::new(AllayData::from(AllayList::from([
                Arc::new(AllayData::from("test")),
                Arc::new(AllayData::from("markdown")),
            ]))),
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
