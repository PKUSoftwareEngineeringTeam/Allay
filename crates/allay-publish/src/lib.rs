mod generate;
use generate::GeneratorWorker;
use std::sync::OnceLock;

/// Start the publish worker.
pub fn start() {
    pub static GENERATOR: OnceLock<GeneratorWorker> = OnceLock::new();
    GENERATOR.get_or_init(|| GeneratorWorker::create()).start();
}
