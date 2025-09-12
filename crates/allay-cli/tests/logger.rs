use allay_cli::logger_init;
use tracing::{error, info, warn};

#[test]
fn test_duplicate_init() {
    logger_init();
    logger_init();
}

#[test]
fn test_logger() {
    logger_init();
    info!("This is an info message");
    warn!("This is a warning message");
    error!("This is an error message");
}

#[test]
#[should_panic(expected = "this is a test panic")]
fn test_panic() {
    logger_init();
    panic!("this is a test panic");
}
