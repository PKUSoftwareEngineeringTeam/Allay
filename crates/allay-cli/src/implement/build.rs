use allay_base::config::BuildArgs;
use tracing::instrument;

/// CLI Build Command
#[instrument(name = "building the site", skip(_args))]
pub fn build(_args: &BuildArgs) -> anyhow::Result<()> {
    println!("Building the site...");
    Ok(())
}
