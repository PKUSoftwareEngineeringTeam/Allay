use allay_backend::ServerResult;
use allay_backend::server::Server;

fn main() -> ServerResult<()> {
    let server = Server::new("./static", 8080, "127.0.0.1".to_string());
    server.serve()?;
    Ok(())
}
