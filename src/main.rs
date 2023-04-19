// use log::{debug, error, info, warn};

mod configloader;
mod gitlabapi;
mod mailsender;

use configloader::prelude::*;
use gitlabapi::prelude::*;

// Just a generic Result type to ease error handling for us. Errors in multithreaded
// async contexts needs some extra restrictions
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn app() -> Result<()> {
    let config = match Config::load_config() {
        Ok(conf) => conf,
        Err(err) => panic!("Error loading configurations. {}", err),
    };

    Ok(())
}

fn main() {
    env_logger::init();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    match rt.block_on(app()) {
        Ok(_) => {}
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
    #[ignore = "it'll show configuration"]
    fn test_show_conf() {
        // init();
        debug!("Current config is {:?}", Config::load_config().unwrap());
    }

    #[tokio::test]
    async fn test_multi_pipelines() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config);

        let multi_jobs = api.get_jobs_by_proj_and_pipeline(JobScope::Manual).await;

        multi_jobs.iter()
            .for_each(|(proj, pipes)| {
                debug!("On project {}", proj);
                let mut pipe_key: Vec<u64> = pipes.clone().into_keys().collect();

                pipe_key.sort();
                pipe_key.reverse();

                pipe_key.iter()
                    .skip(1)
                    .for_each(|pipeid| {
                        debug!("Cancel pipeline {} ?", pipeid);
                    });
                

            })
        
    }
}
