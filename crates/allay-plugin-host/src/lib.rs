use component::Plugin;
pub use component::wit::route::{Header, Method, Request, Response};
use std::path::Path;
use std::sync::Arc;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxView, WasiView};

mod component;

#[derive(Default)]
struct PluginState {
    table: ResourceTable,
    ctx: WasiCtx,
}

impl PluginState {
    fn with_dir(dir: &Path) -> Self {
        let ctx = WasiCtx::builder()
            .preopened_dir(dir, ".", DirPerms::all(), FilePerms::all())
            .unwrap()
            .build();
        Self {
            table: ResourceTable::new(),
            ctx,
        }
    }
}

impl WasiView for PluginState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

pub struct PluginHost {
    store: Store<PluginState>,
    plugin: Arc<Plugin>,
}

impl PluginHost {
    pub fn new(wasm_path: &Path, working_dir: &Path) -> wasmtime::Result<Self> {
        let engine = Engine::new(&Config::default())?;
        let component = Component::from_file(&engine, wasm_path)?;
        let mut store = Store::new(&engine, PluginState::with_dir(working_dir));

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

        let plugin = Plugin::instantiate(&mut store, &component, &linker)?;
        plugin.call_init_plugin(&mut store)?;
        let plugin = Arc::new(plugin);

        let host = Self { store, plugin };
        Ok(host)
    }

    pub fn plugin_name(&mut self) -> wasmtime::Result<String> {
        self.plugin.call_name(&mut self.store)
    }

    pub fn plugin_version(&mut self) -> wasmtime::Result<String> {
        self.plugin.call_version(&mut self.store)
    }
}
