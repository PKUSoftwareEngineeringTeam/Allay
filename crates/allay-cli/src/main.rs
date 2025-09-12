use allay_base::config::GlobalConfigs;

fn main() {
    GlobalConfigs::init().unwrap();

    // execute commands based on CLI input
    // let cli = GlobalConfigs::cli();
}
