use allay_base::config::{ServeArgs, get_allay_config};
use allay_base::file;
use allay_web::server::Server;
use tracing::instrument;

/// CLI Server Command
#[instrument(name = "serving the site", skip(args))]
pub fn serve(args: &ServeArgs) -> anyhow::Result<()> {
    let url = format!("http://{}:{}", args.address, args.port);

    println!("Starting the site server at {}", url);
    allay_publish::start();

    if args.open {
        webbrowser::open(&url).unwrap_or_else(|_| println!("Failed to open the browser"));
    }

    let server = Server::new(
        file::workspace(get_allay_config().publish.dir.as_str()),
        args.port,
        args.address.clone(),
    );
    server.serve()?;
    Ok(())
}
