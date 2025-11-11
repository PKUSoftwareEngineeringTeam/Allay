pub use crate::component::compiler::exports::allay::plugin::compiler::FileType;
pub use crate::component::route::exports::allay::plugin::route::{
    Error as ResponseError, Method, Response,
};
use std::path::Path;
use wasmtime::component::{Component, Linker, ResourceTable};
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

struct Instance {
    base: component::base::Plugin,
    compiler: component::compiler::Plugin,
    route: component::route::Plugin,
}

pub struct PluginHost {
    store: Store<PluginState>,
    instance: Instance,
}

impl PluginHost {
    pub fn new(wasm_path: impl AsRef<Path>) -> wasmtime::Result<Self> {
        let config = Config::default();
        let engine = Engine::new(&config)?;
        let component = Component::from_file(&engine, wasm_path)?;
        let mut store = Store::new(&engine, PluginState::default());

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

        let base = component::base::Plugin::instantiate(&mut store, &component, &linker)?;
        let compiler = component::compiler::Plugin::instantiate(&mut store, &component, &linker)?;
        let route = component::route::Plugin::instantiate(&mut store, &component, &linker)?;

        let instance = Instance {
            base,
            compiler,
            route,
        };

        instance.base.call_init_plugin(&mut store)?;

        let host = Self { store, instance };
        Ok(host)
    }

    pub fn plugin_name(&mut self) -> wasmtime::Result<String> {
        self.instance.base.call_name(&mut self.store)
    }

    pub fn plugin_version(&mut self) -> wasmtime::Result<String> {
        self.instance.base.call_version(&mut self.store)
    }

    pub fn before_compile(
        &mut self,
        source: &str,
        file_type: FileType,
    ) -> wasmtime::Result<String> {
        self.instance.compiler.allay_plugin_compiler().call_before_compile(
            &mut self.store,
            source,
            file_type,
        )
    }

    pub fn after_compile(&mut self, source: &str, file_type: FileType) -> wasmtime::Result<String> {
        self.instance.compiler.allay_plugin_compiler().call_after_compile(
            &mut self.store,
            source,
            file_type,
        )
    }

    pub fn handle_request(
        &mut self,
        method: Method,
        url: &str,
        body: Option<Vec<u8>>,
    ) -> wasmtime::Result<Option<Result<Response, ResponseError>>> {
        self.instance.route.allay_plugin_route().call_handle_request(
            &mut self.store,
            method,
            url,
            body.as_deref(),
        )
    }
}
