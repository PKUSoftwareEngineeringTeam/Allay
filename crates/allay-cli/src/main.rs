use allay_cli::{cli_execute, cli_start, logger_init};

fn main() {
    logger_init();
    cli_start();
    cli_execute();
}
