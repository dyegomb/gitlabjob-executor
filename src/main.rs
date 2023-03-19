use log::{debug, warn, info};

mod load_config;

fn main() {
    env_logger::init();

    debug!("Isso é um debug");
    warn!("Isso é um warning");
    info!("Isso é um info");

    println!("Hello, world!");
}
