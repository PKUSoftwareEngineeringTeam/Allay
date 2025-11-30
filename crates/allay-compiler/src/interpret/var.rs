use crate::ast::GetField;
use crate::interpret::traits::{DataProvider, Variable};
use crate::meta::get_meta;
use crate::{InterpretResult, magic};
use allay_base::config::{CLICommand, get_allay_config, get_cli_config, get_site_config};
use allay_base::data::{AllayData, AllayList};
use allay_base::file;
use allay_base::log::NoPanicUnwrap;
use allay_base::sitemap::SiteMap;
#[cfg(feature = "plugin")]
use allay_plugin::{Plugin, PluginManager};
#[cfg(feature = "plugin")]
use std::cmp;
#[cfg(feature = "plugin")]
use std::process::exit;
#[cfg(feature = "plugin")]
use std::rc::Rc;
use std::sync::atomic::{self, AtomicU32};
use std::sync::{Arc, OnceLock, RwLock};

/// The global site variable, usually from site config
#[derive(Debug)]
pub struct SiteVar {
    pub data: Arc<AllayData>,
}

impl SiteVar {
    pub fn get_instance() -> &'static SiteVar {
        static INSTANCE: OnceLock<SiteVar> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let mut data = get_site_config()
                .get("params")
                .map(|params| {
                    params.as_obj().expect_("Site params should be an obj").as_ref().clone()
                })
                .unwrap_or_default();

            let base_url = if get_cli_config().online {
                // In online mode, use the base_url from site config
                get_site_config()
                    .get("base_url")
                    .expect_("base_url not found in online mode")
                    .as_str()
                    .expect_("base_url should be a string")
                    .clone()
            } else if let CLICommand::Serve(args) = &get_cli_config().command {
                // In serve mode, use the local address and port
                format!("http://{}:{}/", args.address, args.port)
            } else {
                String::new()
            };

            data.insert(magic::BASE_URL.into(), Arc::new(AllayData::from(base_url)));

            let data = Arc::new(data.into());
            SiteVar { data }
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
    cache_version: AtomicU32,
    data: RwLock<Arc<AllayData>>,
}

impl PagesVar {
    pub fn get_instance() -> &'static PagesVar {
        static INSTANCE: OnceLock<PagesVar> = OnceLock::new();
        let instance = INSTANCE.get_or_init(|| PagesVar {
            cache_version: AtomicU32::new(u32::MAX),
            data: RwLock::new(Arc::new(AllayList::new().into())),
        });
        instance.update();
        instance
    }

    #[cfg(feature = "plugin")]
    fn sort_page_var(data: AllayData) -> AllayData {
        let plugin_manager = PluginManager::instance();
        let plugins = plugin_manager.plugins();
        let enabled_plugin: Vec<_> = plugins
            .iter()
            .filter(|plugin| {
                let mut plugin = plugin.lock().expect_("poisoned lock");
                plugin.sort_enabled().unwrap_or(false)
            })
            .collect();

        if enabled_plugin.len() > 1 {
            eprintln!("Error: multiple sort plugins enabled, only one is allowed");
            exit(1);
        }

        if enabled_plugin.is_empty() {
            return data;
        }

        let plugin = enabled_plugin[0].clone();

        struct SortKey {
            json: Rc<String>,
            plugin: Plugin,
        }

        impl PartialEq for SortKey {
            fn eq(&self, other: &Self) -> bool {
                self.json == other.json
            }
        }

        impl Eq for SortKey {}

        impl Ord for SortKey {
            fn cmp(&self, other: &Self) -> cmp::Ordering {
                let mut plugin = self.plugin.lock().expect_("poisoned lock");
                plugin.get_sort_order(&self.json, &other.json).unwrap()
            }
        }

        impl PartialOrd for SortKey {
            fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        if let AllayData::List(list) = data {
            let mut list: Vec<_> = list
                .iter()
                .cloned()
                .map(|item| (item.clone(), Rc::new(item.to_json())))
                .collect();
            list.sort_by_key(|(_, json)| SortKey {
                json: json.clone(),
                plugin: plugin.clone(),
            });
            AllayData::List(Arc::new(list.into_iter().map(|(item, _)| item).collect()))
        } else {
            eprintln!("Error: sort plugin enabled but data is not a list");
            exit(1);
        }
    }

    pub fn update(&self) {
        // see the site map version to decide whether to update
        let version = SiteMap::read().version();
        if self.cache_version.load(atomic::Ordering::SeqCst) == version {
            return;
        }
        self.cache_version.store(version, atomic::Ordering::SeqCst);

        let dir = file::workspace(&get_allay_config().content_dir);
        // walk through the content directory and get all markdown/html files
        if let Ok(entries) = file::read_dir_all_files(&dir) {
            let data = entries
                .into_iter()
                .filter_map(|e| get_meta(e).ok())
                .filter(|meta| meta.get(magic::HIDDEN) != Some(&Arc::new(true.into())))
                .map(AllayData::from)
                .map(Arc::new)
                .collect::<AllayList>()
                .into();

            #[cfg(feature = "plugin")]
            let data = Self::sort_page_var(data);

            *self.data.write().unwrap() = Arc::new(data);
        }
    }
}

impl DataProvider for PagesVar {
    fn get_data(&self) -> Arc<AllayData> {
        self.data.read().unwrap().clone()
    }
}

impl Variable for PagesVar {}

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
