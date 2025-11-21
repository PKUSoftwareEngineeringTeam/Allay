use std::process::exit;

pub fn show_error(msg: &str) -> ! {
    println!("Error: {}", msg);
    exit(1)
}

pub fn show_warning(msg: &str) {
    println!("Warning: {}", msg);
}
