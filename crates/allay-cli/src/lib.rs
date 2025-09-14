mod implement;
pub mod initialize;

use allay_base::config::{AllayCLI, CLI_CONFIG, CLICommand};
use implement::*;

pub fn execute() -> anyhow::Result<()> {
    initialize::initialize()?;
    execute_cli(&CLI_CONFIG)
}

pub fn execute_cli(cli: &AllayCLI) -> anyhow::Result<()> {
    match &cli.command {
        CLICommand::New(args) => new(args),
        CLICommand::Init(args) => init(args),
        CLICommand::Build(args) => build(args),
        CLICommand::Server(args) => server(args),
    }
}
