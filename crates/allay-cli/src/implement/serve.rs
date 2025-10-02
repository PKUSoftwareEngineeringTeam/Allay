use allay_base::config::{ServeArgs, get_allay_config};
use allay_base::file;
use allay_web::server::Server;
use tracing::instrument;

/// CLI Server Command
#[instrument(name = "serving the site", skip(args))]
pub fn serve(args: &ServeArgs) -> anyhow::Result<()> {
    println!("Starting the site server...");
    allay_publish::start();

    let server = Server::new(
        file::workspace(get_allay_config().publish.dir.as_str()),
        args.port,
        args.address.clone(),
    );
    server.serve()?;
    Ok(())
}
