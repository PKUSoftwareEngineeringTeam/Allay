pub use clap::Parser;
use clap::{Args, Subcommand};
use std::sync::OnceLock;

// NOTE: The doc comments here will be used by clap for the CLI help messages

/// An easy and configurable blog engine
#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(long_about = "Named after the Minecraft 'Allay' mob")]
#[command(propagate_version = true)]
pub struct AllayCLI {
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

pub static CLI_CONFIG: OnceLock<AllayCLI> = OnceLock::new();

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new Allay site in the current directory
    Init(InitArgs),
    /// Build all the contents and publish it to the output directory
    Build(BuildArgs),
    /// Start the embedded server to preview the site
    Server(ServerArgs),
}

#[derive(Args, Debug)]
pub struct InitArgs {}

#[derive(Args, Debug)]
pub struct BuildArgs {}

#[derive(Args, Debug)]
pub struct ServerArgs {
    /// Port to listen on
    #[arg(short, long, default_value_t = 8000)]
    port: u16,

    /// Address to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    address: String,

    /// Use the baseUrl from the config file
    #[arg(short, long)]
    base_url: bool,
}
