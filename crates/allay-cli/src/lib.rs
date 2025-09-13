use allay_base::{
    config::cli::*,
    config::site::*,
    costants::{CONFIG_FILE, CONTENT_DIR, OUTPUT_DIR, STATIC_DIR, THEMES_DIR},
    file,
};

pub fn cli_start() {
    CLI_CONFIG.set(AllayCLI::parse()).expect("Duplicate initialization");
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
