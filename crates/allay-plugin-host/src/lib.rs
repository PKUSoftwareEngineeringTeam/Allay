use component::Plugin;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{AsContextMut, Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

mod component;

#[derive(Default)]
struct PluginState {
    table: ResourceTable,
    ctx: WasiCtx,
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
    store: Arc<Mutex<Store<PluginState>>>,
    plugin: Arc<Plugin>,
}

impl PluginHost {
    pub fn new(wasm_path: impl AsRef<Path>) -> wasmtime::Result<Self> {
        let engine = Engine::new(&Config::default())?;
        let component = Component::from_file(&engine, wasm_path)?;
        let mut store = Store::new(&engine, PluginState::default());

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

        let plugin = Plugin::instantiate(&mut store, &component, &linker)?;
        let plugin = Arc::new(plugin);
        let store = Arc::new(Mutex::new(store));

        let host = Self { store, plugin };
        Ok(host)
    }

    pub fn plugin_name(&self) -> wasmtime::Result<String> {
        let mut store = self.store.blocking_lock();
        self.plugin.call_name(store.as_context_mut())
    }

    pub fn plugin_version(&self) -> wasmtime::Result<String> {
        let mut store = self.store.blocking_lock();
        self.plugin.call_version(store.as_context_mut())
    }
}
