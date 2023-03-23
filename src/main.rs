use log::{debug, info, warn};
use crate::load_config::load_config;

mod load_config;
mod mail_sender;

fn main() {
    env_logger::init();

    let config = load_config().unwrap();


    debug!("Isso é um debug");
    warn!("Isso é um warning");
    info!("Isso é um info");

    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_show_conf() {
        // println!("Current config is {:?}", load_config().unwrap());

        


    }
}
