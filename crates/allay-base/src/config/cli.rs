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
    /// Increase output verbosity (use -vv for more, up to -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Specify the root directory of the Allay site (default: current directory)
    #[arg(short, long, global = true)]
    pub root: Option<String>,

    /// Serve the site in online mode (which means it will use the base_url in the config)
    #[arg(long)]
    pub online: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: CLICommand,
}

pub fn get_cli_config() -> &'static AllayCLI {
    static INSTANCE: OnceLock<AllayCLI> = OnceLock::new();
    INSTANCE.get_or_init(AllayCLI::parse)
}

#[derive(Subcommand, Debug)]
pub enum CLICommand {
    /// Create a new Allay site in the specified directory
    New(NewArgs),
    /// Initialize a new Allay site in the current directory
    Init(InitArgs),
    /// Build all the contents and publish it to the output directory
    Build(BuildArgs),
    /// Start the embedded server to preview the site
    Serve(ServeArgs),
}

#[derive(Args, Debug)]
pub struct NewArgs {
    /// Directory to create the new site in
    pub dir: String,
    /// Skip cloning the default theme
    #[arg(long, default_value_t = false)]
    pub skip_theme: bool,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Skip cloning the default theme
    #[arg(long, default_value_t = false)]
    pub skip_theme: bool,
}

#[derive(Args, Debug)]
pub struct BuildArgs {}

#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Port to listen on
    #[arg(short, long, default_value_t = 8000)]
    pub port: u16,

    /// Address to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    pub address: String,

    /// Use the baseUrl from the config file
    #[arg(short, long)]
    pub base_url: bool,

    /// Open the site in the browser
    #[arg(long)]
    pub open: bool,
}
