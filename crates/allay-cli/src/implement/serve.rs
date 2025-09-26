use allay_base::config::ServerArgs;
use std::time::Duration;
use tracing::instrument;

/// CLI Server Command
#[instrument(name = "serving the site", skip(_args))]
pub fn server(_args: &ServerArgs) -> anyhow::Result<()> {
    println!("Starting the site server...");
    allay_publish::start();
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
