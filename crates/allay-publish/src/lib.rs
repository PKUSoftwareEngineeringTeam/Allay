mod generator;
mod worker;

use std::sync::OnceLock;
use worker::GeneratorWorker;

/// Start the publishing worker.
pub fn start() {
    pub static GENERATOR: OnceLock<GeneratorWorker> = OnceLock::new();
    GENERATOR.get_or_init(GeneratorWorker::create).start();
}

/// Generate all files once.
pub fn generate_once() {
    GeneratorWorker::create().generate_once();
}
