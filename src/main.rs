//! It's a workaround until conclusion of <https://gitlab.com/gitlab-org/gitlab/-/issues/17718>,
//! you can create manual jobs that would be started by this program.
//!
//! Its proposal is to execute manual jobs inside a Gitlab group or project, so you can queue a
//! manual job that will be started in a proper time by this program.
//!
//! ## How to use
//! Basically you have to feed the _.env_[^note] file as example below.
//!
//! [^note]: You can change file name to read with the environment variable *`ENV_FILE`*.
//!  
//! ```
//! private_token="XXXXXXXXXXXXX"
//! base_url="https://gitlab.com/"
//! project_id=123
//! group_id=1
//! production_tag_key="PROD_TAG"
//!
//! [smtp]
//! server="mail.com"
//! user="user"
//! from="user@mail.com"
//! to="destination@mail.com"
//! subject="[Subject Prefix] "
//! pass="Secret"
//! ```
//!
//! It also supports definition from environment variables, whom **takes precedence**.
//!
//! The SMTP section is only needed if you want to receive report emails.
//! SMTP settings from environment variables must has `SMTP_` prefix.
//!

/// Get configuration settings from environment variables and/or toml file.
mod configloader;

/// API tools and the actual Gitlab API caller
mod gitlabapi;

/// Module to support mail reports
mod mailsender;

use configloader::prelude::*;
use gitlabapi::prelude::*;
use mailsender::prelude::*;

/// Just a generic Result type to ease error handling for us. Errors in multithreaded
/// async contexts needs some extra restrictions
///
/// Reference: <https://blog.logrocket.com/a-practical-guide-to-async-in-rust/>
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// The actual code to run
async fn app() -> Result<()> {
    let config = match Config::load_config() {
        Ok(conf) => conf,
        Err(err) => panic!("Error loading configurations. {}", err),
    };

    Ok(())
}

/// Load tokio runtime
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
    async fn test_multiple_pipelines() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config);

        let multi_jobs = api.get_jobs_by_proj_and_pipeline(JobScope::Manual).await;

        multi_jobs.iter().for_each(|(proj, pipes)| {
            debug!("On project {}", proj);
            let mut pipe_key: Vec<u64> = pipes.clone().into_keys().collect();

            pipe_key.sort();
            pipe_key.reverse();

            pipe_key.iter().skip(1).for_each(|pipeid| {
                debug!("Cancel pipeline {} ?", pipeid);
            });
        })
    }

    #[tokio::test]
    async fn test_stream_next() {
        let stream = futures::stream::iter(1..=200)
            .map(|number| async move {
                println!("Start task: {}", number);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                number
            })
            .buffer_unordered(40)
            .fuse();
        tokio::pin!(stream);

        let mut feed: Vec<usize> = vec![];
        while let Some(num) = stream.next().await {
            feed.push(num);
            println!("Done task: {}", num);
        }

        assert_eq!(200, feed.len());
        assert_eq!(20100, feed.iter().sum::<usize>());
    }

    #[tokio::test]
    async fn test_stream_collect() {
        let feed: Vec<usize> = futures::stream::iter(1..=200)
            .map(|number| async move {
                println!("Start task: {}", number);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                number
            })
            .buffer_unordered(40)
            .collect()
            .await;

        assert_eq!(feed.len(), 200);
        assert_eq!(20100, feed.iter().sum::<usize>());
    }

    #[tokio::test]
    async fn test_check_source_tag() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config);

        let proj_jobs = api.get_jobs_by_project(JobScope::Manual).await;

        let tagged_jobs: Vec<JobInfo> = proj_jobs
            .values()
            .flat_map(|jobs| jobs.to_vec())
            .filter(|job| job.git_tag.is_some())
            .filter(|tagged_job| {
                api.get_proj_git_tags(tagged_job.source_id.unwrap()).await.contains(&tagged_job.git_tag.unwrap())
            })
            .collect();

        // let job_and_tags

        // jobs.iter()
        //     .for_each(|(proj, jobs)| {
        //         jobs.iter()
        //             .for_each(|job| {
        //                 if let Some(git_tag) = &job.git_tag {
        //                     if let Some(source_id) = job.source_id {
        //                         let source_tags = api.get_proj_git_tags(source_id);
        //                     }
        //                 };
        //             });
        //     })
    }
}
