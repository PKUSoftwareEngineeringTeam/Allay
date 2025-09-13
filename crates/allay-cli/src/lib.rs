use allay_base::{
    config::cli::*,
    config::site::*,
    costants::{CONFIG_FILE, CONTENT_DIR, OUTPUT_DIR, STATIC_DIR, THEMES_DIR},
    file,
};

pub fn cli_execute() -> anyhow::Result<()> {
    match &CLI_CONFIG.command {
        Commands::New(args) => new(args),
        Commands::Init(args) => init(args),
        Commands::Build(args) => build(args),
        Commands::Server(args) => server(args),
    }
}

fn new(args: &NewArgs) -> anyhow::Result<()> {
    let dir = &args.dir;
    file::create_dir_recursively(file::workspace(dir))?;

    file::create_dir(file::workspace_sub(CONTENT_DIR, dir))?;
    file::create_dir(file::workspace_sub(OUTPUT_DIR, dir))?;
    file::create_dir(file::workspace_sub(THEMES_DIR, dir))?;
    file::create_dir(file::workspace_sub(STATIC_DIR, dir))?;
    file::write_file(file::workspace_sub(CONFIG_FILE, dir), DEFAULT_SITE_CONFIG)?;
    Ok(())
}

fn init(_args: &InitArgs) -> anyhow::Result<()> {
    new(&NewArgs { dir: ".".into() })
}

fn build(_args: &BuildArgs) -> anyhow::Result<()> {
    Ok(())
}

fn server(_args: &ServerArgs) -> anyhow::Result<()> {
    Ok(())
}
