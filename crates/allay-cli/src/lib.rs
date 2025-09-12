use allay_base::{
    config::cli::*,
    config::site::*,
    costants::{CONFIG_FILE, CONTENT_DIR, OUTPUT_DIR, STATIC_DIR, THEMES_DIR},
    file,
};
use std::panic;
use tracing::{Level, span};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt};

use std::sync::Once;

static LOGGER_INIT: Once = Once::new();

pub fn logger_init() {
    LOGGER_INIT.call_once(|| {
        let fmt_layer = tracing_subscriber::fmt::layer().pretty();
        let tree_layer = tracing_tree::HierarchicalLayer::new(2);

        let subscriber = tracing_subscriber::registry()
            .with(fmt_layer)
            .with(tree_layer)
            .with(EnvFilter::from_default_env());

        tracing::subscriber::set_global_default(subscriber).unwrap();
    });

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        tracing::error!("panic occurred: {panic_info}");
        original_hook(panic_info);
    }));
}

pub fn cli_start() {
    CLI_CONFIG
        .set(AllayCLI::parse())
        .expect("Duplicate initialization");
}

pub fn cli_execute() {
    let cli = CLI_CONFIG.get().expect("CLI not initialized");
    match &cli.command {
        Commands::Init(args) => init(args),
        Commands::Build(args) => build(args),
        Commands::Server(args) => server(args),
    }
}

fn init(_args: &InitArgs) {
    let span = span!(Level::INFO, "Initializing Allay site directories");
    let _enter = span.enter();

    file::create_dir(file::workspace(CONTENT_DIR)).unwrap();
    file::create_dir(file::workspace(OUTPUT_DIR)).unwrap();
    file::create_dir(file::workspace(THEMES_DIR)).unwrap();
    file::create_dir(file::workspace(STATIC_DIR)).unwrap();
    file::write_file(file::workspace(CONFIG_FILE), DEFAULT_SITE_CONFIG).unwrap()
}

fn build(_args: &BuildArgs) {
    load_site_config();
}

fn server(_args: &ServerArgs) {
    load_site_config();
}
