use component::Plugin;
use std::path::Path;
use std::sync::Arc;
use wasmtime::component::{Component, Instance, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
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
    store: Store<PluginState>,
    plugin: Arc<Plugin>,
    instance: Instance,
}

impl PluginHost {
    pub fn new(wasm_path: impl AsRef<Path>) -> wasmtime::Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true).async_support(true);

        let engine = Engine::new(&config)?;
        let component = Component::from_file(&engine, wasm_path)?;
        let mut store = Store::new(&engine, PluginState::default());

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;

        let instance = linker.instantiate(&mut store, &component)?;

        let plugin = Plugin::new(&mut store, &instance)?;
        plugin.call_init_plugin(&mut store)?;
        let plugin = Arc::new(plugin);

        let host = Self {
            store,
            plugin,
            instance,
        };
        Ok(host)
    }

    pub fn plugin_name(&mut self) -> wasmtime::Result<String> {
        self.plugin.call_name(&mut self.store)
    }

    pub fn plugin_version(&mut self) -> wasmtime::Result<String> {
        self.plugin.call_version(&mut self.store)
    }
}
