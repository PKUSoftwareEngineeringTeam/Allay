mod generate;
use generate::GeneratorWorker;
use std::sync::OnceLock;

/// Start the publishing worker.
pub fn start() {
    pub static GENERATOR: OnceLock<GeneratorWorker> = OnceLock::new();
    GENERATOR.get_or_init(GeneratorWorker::create).start();
}

/// Generate all files once.
pub fn generate_once() {
    GeneratorWorker::create().generate_once();
}
