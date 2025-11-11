pub mod base {
    wasmtime::component::bindgen!({
        path: "../allay-plugin-api/wit/base.wit",
        world: "plugin"
    });
}
pub mod compiler {
    wasmtime::component::bindgen!({
        path: "../allay-plugin-api/wit/compiler.wit",
        world: "plugin"
    });
}
pub mod route {
    wasmtime::component::bindgen!({
        path: "../allay-plugin-api/wit/route.wit",
        world: "plugin"
    });
}
