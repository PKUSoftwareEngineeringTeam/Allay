use component::Plugin;
pub use component::exports::allay::plugin::compiler::FileType;
pub use component::exports::allay::plugin::route::{Method, Request, Response};
use std::path::Path;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

mod component;
// mod temp;

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
    instance: Plugin,
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

        let instance = Plugin::instantiate(&mut store, &component, &linker)?;
        instance.call_init_plugin(&mut store)?;

        let host = Self { store, instance };
        Ok(host)
    }

    pub fn plugin_name(&mut self) -> wasmtime::Result<String> {
        self.instance.call_name(&mut self.store)
    }

    pub fn plugin_version(&mut self) -> wasmtime::Result<String> {
        self.instance.call_version(&mut self.store)
    }

    pub fn before_compile(
        &mut self,
        source: &str,
        file_type: FileType,
    ) -> wasmtime::Result<String> {
        self.instance.allay_plugin_compiler().call_before_compile(
            &mut self.store,
            source,
            file_type,
        )
    }

    pub fn after_compile(&mut self, source: &str, file_type: FileType) -> wasmtime::Result<String> {
        self.instance
            .allay_plugin_compiler()
            .call_after_compile(&mut self.store, source, file_type)
    }

    pub async fn handle_request(&mut self, _request: Request) -> wasmtime::Result<Response> {
        // self.instance.allay_plugin_route().call_handle(, request).await
        todo!()
    }
}
