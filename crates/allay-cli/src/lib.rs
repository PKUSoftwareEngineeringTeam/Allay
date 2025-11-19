mod implement;
pub mod initialize;

use allay_base::config::{AllayCLI, CLICommand, get_cli_config};
use implement::*;

pub async fn execute() -> anyhow::Result<()> {
    initialize::initialize()?;
    execute_cli(get_cli_config()).await
}

pub async fn execute_cli(cli: &AllayCLI) -> anyhow::Result<()> {
    match &cli.command {
        CLICommand::New(args) => new(args),
        CLICommand::Init(args) => init(args),
        CLICommand::Build(args) => build(args),
        CLICommand::Serve(args) => serve(args).await,
    }
}
