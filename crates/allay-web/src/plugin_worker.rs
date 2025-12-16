use allay_base::lock;
use allay_plugin::Plugin;
use allay_plugin::types::{Request, Response, response_internal_error};

pub struct PluginWorker {
    pool: rayon::ThreadPool,
}

impl PluginWorker {
    pub fn new(num_workers: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_workers)
            .build()
            .expect("Failed to build thread pool");
        Self { pool }
    }

    pub fn handle_request(&self, plugin: Plugin, request: Request) -> Response {
        self.pool.install(move || {
            let mut plugin = lock!(plugin);
            plugin.handle_request(request).unwrap_or(response_internal_error())
        })
    }
}
