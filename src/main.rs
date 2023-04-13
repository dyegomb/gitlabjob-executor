use log::{debug, info, warn, error};
use crate::load_config::Config;

mod load_config;
mod mailsender;
mod gitlabapi;

// static PRODUCTION_KEY_TAG: &str = "PROD_TAG";

// Just a generic Result type to ease error handling for us. Errors in multithreaded
// async contexts needs some extra restrictions
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn app() -> Result<()> {
    // I treat this as the `main` function of the async part of our program. 
    // todo!()
    println!("Running async tasks");
    Ok(())
}


fn main() {
    env_logger::init();

    let _config = Config::load_config().unwrap();


    debug!("Isso é um debug");
    warn!("Isso é um warning");
    info!("Isso é um info");

    println!("Hello, world!");

    // let mut rt = tokio::runtime::Runtime::new().unwrap();
    // let rt = tokio::runtime::Runtime::new().unwrap();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
        // .block_on(async {
        //     println!("Hello world");
        // });


    match rt.block_on(app()) {
        Ok(_) => info!("Done"),
        Err(e) => error!("An error ocurred: {}", e),
    };

}

#[cfg(test)]
mod test {
    use super::*;

    fn init() {
        let _ = env_logger::builder()
            // Include all events in tests
            .filter_level(log::LevelFilter::max())
            // Ensure events are captured by `cargo test`
            .is_test(true)
            // Ignore errors initializing the logger if tests race to configure it
            .try_init();
    }

    #[test]
    fn test_show_conf() {
        init();
        println!("Current config is {:?}", Config::load_config().unwrap());

        


    }
}
