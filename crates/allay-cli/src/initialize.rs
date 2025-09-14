use allay_base::config::{CLI_CONFIG, CLICommand, ENVRIONMENT, LOG_CONFIG};
use allay_base::file;
use anyhow::Ok;
use std::io;
use tracing::{Level, info_span};
use tracing_subscriber::fmt::format::{FmtSpan, format};

pub fn initialize() -> anyhow::Result<()> {
    init_root()?;
    init_logger()?;

    let span = info_span!("initialize");
    let _enter = span.enter();

    Ok(())
}

pub fn init_root() -> anyhow::Result<()> {
    if let CLICommand::New(args) = &CLI_CONFIG.command {
        let dir = &args.dir;
        if file::dirty_dir(dir)? {
            return Err(anyhow::anyhow!("Directory is not empty"));
        }
        file::create_dir_if_not_exists(dir)?;
        file::set_root(dir);
    } else {
        file::set_root(CLI_CONFIG.root.clone().unwrap_or(".".into()));
    }

    Ok(())
}

pub fn init_logger() -> anyhow::Result<()> {
    let log_dir = &LOG_CONFIG.dir;
    file::create_dir_if_not_exists(file::workspace(log_dir))?;

    let format = format()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    if ENVRIONMENT.is_dev() {
        tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_writer(io::stdout)
            .with_span_events(FmtSpan::ENTER)
            .event_format(format.with_ansi(true))
            .pretty()
            .init();
    } else {
        let file_appender =
            tracing_appender::rolling::minutely(file::workspace(log_dir), "allay.log");

        tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .with_writer(file_appender)
            .event_format(format.with_ansi(false))
            .init();
    }

    Ok(())
}
