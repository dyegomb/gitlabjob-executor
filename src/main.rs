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

    let mail_relay = match &config.smtp {
        Some(smtp) => match smtp.is_valid() {
            true => match MailSender::try_new(smtp.clone()).await {
                Ok(mailer) => mailer.relay,
                Err(error) => {
                    error!("{}", error);
                    None
                }
            },
            false => None,
        },
        None => None,
    };

    // Scan projects for Manual jobs
    let api = GitlabJOB::new(&config);
    let mut playable_jobs: HashSet<&JobInfo> = HashSet::new();
    let mut cancel_jobs: HashSet<&JobInfo> = HashSet::new();

    let multi_jobs = api.get_jobs_by_proj_and_pipeline(JobScope::Manual).await;

    info!("Projects with manual/paused jobs: {:?}", multi_jobs.keys());

    multi_jobs.iter().for_each(|(_, pipe_map)| {
        pipe_map.iter().for_each(|(_, jobs)| {
            jobs.iter().for_each(|job| {
                playable_jobs.insert(job);
            })
        })
    });

    multi_jobs.iter().for_each(|(proj, pipes)| {
        debug!("On project {}", proj);
        let mut pipe_key: Vec<&u64> = pipes.keys().collect();

        pipe_key.sort();
        pipe_key.reverse();

        pipe_key.iter().skip(1).for_each(|pipeid| {
            warn!("Pipeline id to be canceled: {}", pipeid);
            pipes.get(pipeid).iter().for_each(|jobs| {
                jobs.iter().for_each(|job| {
                    if playable_jobs.remove(job) {
                        info!("Job removed from playable list: {}", job)
                    };
                    cancel_jobs.insert(job);
                });
            });
        });
    });

    let cant_cancel = match api.bulk_jobs_cancel(&cancel_jobs).await {
        Ok(_) => HashSet::new(),
        Err(jobs) => {
            jobs.iter().for_each(|job| {
                cancel_jobs.remove(job);
            });
            jobs
        }
    };

    cancel_jobs.iter().for_each(|job| {
        warn!("Job {} canceled due to duplicated pipeline.", job);

        if mail_relay.is_some() {
            debug!("Sending mail report");
            let to = &job.user_mail;
            let mut update_job = (*job).clone();

            update_job.status = Some(JobScope::Canceled);

            let smtp = config.smtp.as_ref().unwrap();
            let body = smtp.body_builder(
                format!("Job {} canceled due to duplicated pipeline.", update_job),
                update_job.to_html(),
                to,
            );

            let _ = mail_relay.as_ref().unwrap().send(&body);
        }
    });

    cant_cancel.iter().for_each(|job| {
        error!("ERROR while cancels the job {}", job);
    });

    cancel_jobs.clear();

    let tagged_jobs: Vec<&JobInfo> = playable_jobs
        .iter()
        .filter(|job| job.git_tag.is_some() && job.source_id.is_some())
        .copied()
        .collect();

    let invalid_tags = stream::iter(tagged_jobs)
        .filter(|job| async {
            let job_tag = &job.git_tag.clone().unwrap();
            !api.get_proj_git_tags(job.source_id.unwrap())
                .await
                .contains(job_tag)
        })
        .fuse();

    tokio::pin!(invalid_tags);

    while let Some(job) = invalid_tags.next().await {
        cancel_jobs.insert(job);
        playable_jobs.remove(job);
    }

    debug!("Jobs to cancel {:?}", cancel_jobs);
    // call_cancel_jobs().await;

    debug!("\n\nPlayable jobs:\n{:?}", playable_jobs);

    // stream::iter(tagged_jobs)
    //     .filter_map(|job| async {
    //         if !api
    //             .get_proj_git_tags(job.source_id.unwrap())
    //             .await
    //             .contains(&job.to_owned().git_tag.unwrap())
    //         {
    //             Some(job)
    //         } else {
    //             None
    //         }
    //     })
    //     .for_each_concurrent(STREAM_BUFF_SIZE, |cancel_job| {
    //         async move {
    //             debug!("Cancel job: {:?}", cancel_job);
    //             // api.cancel_job(cancel_job).await; // remove the "move" in async
    //         }
    //     })
    //     .await;

    Ok(())
}

// async fn cancel_jobs(){}

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

        let api = GitlabJOB::new(&config);

        let multi_jobs = api.get_jobs_by_proj_and_pipeline(JobScope::Manual).await;

        multi_jobs.iter().for_each(|(proj, pipes)| {
            debug!("On project {}", proj);
            let mut pipe_key: Vec<&u64> = pipes.keys().collect();

            pipe_key.sort();
            pipe_key.reverse();

            pipe_key.iter().skip(1).for_each(|pipeid| {
                debug!("Cancel pipeline: {}", pipeid);
                debug!("Its jobs: {:?}", pipes.get(pipeid));
            });
        })
    }

    #[tokio::test]
    async fn test_cancel_job_with_invalid_tags() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(&config);

        let proj_jobs = api.get_jobs_by_project(JobScope::Manual).await;

        let tagged_jobs: Vec<JobInfo> = proj_jobs
            .values()
            .flat_map(|jobs| jobs.to_vec())
            .filter(|job| job.git_tag.is_some() && job.source_id.is_some())
            .collect();

        stream::iter(tagged_jobs)
            .filter_map(|job| async {
                if !api
                    .get_proj_git_tags(job.source_id.unwrap())
                    .await
                    .contains(&job.to_owned().git_tag.unwrap())
                {
                    Some(job)
                } else {
                    None
                }
            })
            .for_each_concurrent(STREAM_BUFF_SIZE, |cancel_job| {
                async move {
                    debug!("Cancel job: {:?}", cancel_job);
                    // api.cancel_job(cancel_job).await; // remove the "move" in async
                }
            })
            .await;
    }
}
