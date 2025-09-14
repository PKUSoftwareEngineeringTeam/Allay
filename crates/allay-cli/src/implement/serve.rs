use allay_base::config::ServerArgs;
use tracing::instrument;

#[instrument(name = "serving the site", skip(_args))]
pub fn server(_args: &ServerArgs) -> anyhow::Result<()> {
    println!("Starting the site server...");

    Ok(())
}
