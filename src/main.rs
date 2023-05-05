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
//! production_tag_key="PROD_TAG" # Variable to look for in a pipeline
//! max_wait_time=1800 # Max waiting time for a job in seconds
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

use tokio::time as tktime;

use configloader::prelude::*;
use gitlabapi::prelude::*;

/// Just a generic Result type to ease error handling for us. Errors in multithreaded
/// async contexts needs some extra restrictions
///
/// Reference: <https://blog.logrocket.com/a-practical-guide-to-async-in-rust/>
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
enum MailReason {
    Duplicated,
    InvalidTag,
    ErrorToCancel,
    ErrorToPlay,
    MaxWaitElapsed,
    Status(JobScope),
}

/// The actual code to run
#[tokio::main]
async fn main() -> Result<()> {
    // Set default log level for INFO, changed with "RUST_LOG"
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

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
    let mut mail_jobs_list: Vec<(&JobInfo, MailReason)> = vec![];

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
            warn!("A duplicated pipeline will be canceled: {}", pipeid);
            pipes.get(pipeid).iter().for_each(|jobs| {
                jobs.iter().for_each(|job| {
                    warn!("Job {} canceled due to duplicated pipeline.", job);
                    playable_jobs.remove(job);
                    cancel_jobs.insert(job);
                    mail_jobs_list.push((job, MailReason::Duplicated))
                });
            });
        });
    });

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
        warn!("Job {} canceled due to have a invalid git tag.", job);
        cancel_jobs.insert(job);
        playable_jobs.remove(job);
        mail_jobs_list.push((job, MailReason::InvalidTag));
    }

    debug!("Jobs to cancel {:?}", cancel_jobs);

    let cant_cancel = match api.bulk_jobs_cancel(&cancel_jobs).await {
        Ok(_) => HashSet::new(),
        Err(jobs) => {
            jobs.iter().for_each(|job| {
                cancel_jobs.remove(job);
            });
            jobs
        }
    };

    cant_cancel.iter().for_each(|job| {
        error!("ERROR to cancel the job {}", job);
        mail_jobs_list.push((job, MailReason::ErrorToCancel))
    });

    debug!("\n\nPlayable jobs:\n{:?}", playable_jobs);

    // Let's play the jobs
    let played_jobs = stream::iter(&playable_jobs)
        .map(|job| async {
            let job = *job;
            info!("Start job {}.", job);
            match api.play_job(job).await {
                Err(_) => (job, MailReason::ErrorToPlay),
                Ok(_) => {
                    async {
                        let mut cur_job_status = api.get_new_job_status(job).await;
                        let pending_status = vec![
                            JobScope::Pending,
                            JobScope::Running,
                            JobScope::WaitingForResource,
                            JobScope::Manual,
                        ];
                        let cronometer = tktime::Instant::now();
                        let duration =
                            tktime::Duration::from_secs(config.max_wait_time.unwrap_or(30));
                        let loop_time = tktime::Duration::from_secs(10);

                        loop {
                            if let Some(status) = cur_job_status {
                                if pending_status.contains(&status) {
                                    debug!("Waiting job {}", job);
                                } else {
                                    break;
                                }
                            }

                            if cronometer.elapsed() >= duration {
                                warn!("Max wait time ended for job {}", job);
                                break;
                            }

                            tktime::sleep(loop_time).await;

                            cur_job_status = api.get_new_job_status(job).await;
                        }

                        let status = cur_job_status.unwrap_or(JobScope::Invalid);
                        let reason = match pending_status.contains(&status) {
                            true => MailReason::MaxWaitElapsed,
                            false => MailReason::Status(status),
                        };

                        (job, reason)
                    }
                    .await
                }
            }
        })
        .buffer_unordered(STREAM_BUFF_SIZE)
        .fuse();

    tokio::pin!(played_jobs);

    while let Some(result) = played_jobs.next().await {
        mail_jobs_list.push(result);
    }

    // Mail reports
    if let (Some(mailer), Some(smtp_trait)) = (mail_relay, config.smtp) {
        let jobs_new_status = stream::iter(&mail_jobs_list)
            .map(|(job, _reason)| async {
                let job_clone = (*job).clone();
                let new_status = api.get_new_job_status(&job_clone).await;

                (job_clone, new_status)
            })
            .buffer_unordered(STREAM_BUFF_SIZE)
            .collect::<HashMap<JobInfo, Option<JobScope>>>()
            .await;

        info!("Sending mail reports.");

        for (job, reason) in mail_jobs_list {
            let subject = match reason {
                MailReason::Duplicated => {
                    format!("Job {} canceled due to duplicated pipeline", job)
                }
                MailReason::InvalidTag => format!("Job {} canceled due to invalid git tag", job),
                MailReason::ErrorToCancel => format!("Error trying to cancel job {}", job),
                MailReason::ErrorToPlay => format!("Error to start job {}", job),
                MailReason::MaxWaitElapsed => format!("Max wait time elapsed for job {}", job),
                MailReason::Status(status) => format!("Status of job {}: {}", job, status),
            };

            let to = &job.user_mail;

            let mut job_updated = job.to_owned();

            if let Some(job_status) = jobs_new_status.get(job) {
                if job.status != *job_status {
                    job_updated.status = *job_status;
                }
            };

            let body = smtp_trait.body_builder(subject, job_updated.to_html(), to);

            let _ = mailer.send(&body);
        }
    }

    info!("Done");
    Ok(())
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
}
