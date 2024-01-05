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
//! production_tag_key="PROD_TAG" # Variable to search in a pipeline
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

// /// Get configuration settings from environment variables and/or toml file.
// mod configloader;

// /// API tools and the actual Gitlab API caller
// mod gitlabapi;

// /// Module to support mail reports
// mod mailsender;
use futures::stream::{self, StreamExt};
use log::{error, info};
// use std::collections::{HashMap, HashSet};
use tokio::time as tktime;
// use tokio_stream::StreamExt;

use configloader::prelude::*;
use gitlabapi::{prelude::*, setters::JobActions};
use mailsender::prelude::*;

mod tests;
mod utils;

/// Just a generic Result type to ease error handling for us. Errors in multithreaded
/// async contexts needs some extra restrictions
///
/// Reference: <https://blog.logrocket.com/a-practical-guide-to-async-in-rust/>
// type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
enum MailReason {
    Duplicated,
    InvalidTag,
    ErrorToCancel,
    ErrorToPlay,
    MaxWaitElapsed,
    Status(JobScope),
}

#[tokio::main]
async fn main() {
    // Set default log level for INFO, changed with "RUST_LOG" environment variable
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = match Config::load_config() {
        Ok(conf) => conf,
        Err(err) => panic!("Error loading configurations. {}", err),
    };

    let mail_relay_handle = tokio::spawn(utils::mailrelay_buid(config.clone()));

    // Scan projects for Manual jobs
    let api = GitlabJOB::new(&config);

    let proj_jobs = match config.group_id {
        Some(group_id) => api.get_jobs(GroupID(group_id), JobScope::Manual).await,
        None => match config.project_id {
            Some(proj_id) => api.get_jobs(ProjectID(proj_id), JobScope::Manual).await,
            None => panic!("There's no project to scan for jobs."),
        },
    };

    log::info!(
        "Projects with {} status jobs: {:?}",
        JobScope::Manual,
        proj_jobs.keys()
    );

    let verified_jobs = utils::validate_jobs(&api, &proj_jobs).await;

    let actions = stream::iter(&verified_jobs)
        .map(|(job, context)| match context.0 {
            true => api.play_job(job),
            false => api.cancel_job(job),
        })
        .buffer_unordered(STREAM_BUFF_SIZE)
        .fuse();
    tokio::pin!(actions);

    let mail_relay = mail_relay_handle.await.unwrap_or_default();
    let smtp_configs = config.smtp.clone().unwrap_or_default();
    let mut mailing_handlers = vec![];
    let pending_status = [
        JobScope::Pending,
        JobScope::Running,
        JobScope::WaitingForResource,
        JobScope::Manual,
    ];

    while let Some(result) = actions.next().await {
        match result {
            Ok(job) => {
                let cronometer = tktime::Instant::now();
                let max_wait = tktime::Duration::from_secs(config.max_wait_time.unwrap_or(30));
                let loop_wait_time = tktime::Duration::from_secs(10);

                loop {
                    let curr_status = api.get_status(job).await;

                    if pending_status.contains(&curr_status) {
                        tktime::sleep(loop_wait_time).await;
                    } else {
                        if let Some(mailer) = mail_relay {
                            let job = job.clone();
                            mailing_handlers.push(tokio::spawn(async move {
                                mailer.send(&utils::mail_message(
                                    &job,
                                    MailReason::Status(curr_status),
                                    &smtp_configs,
                                ))
                            }));
                        } else {
                            info!("Job {} finished with status: {}", job, curr_status);
                        }

                        break;
                    }

                    if cronometer.elapsed() >= max_wait {
                        if let Some(mailer) = mail_relay {
                            let job = job.clone();
                            mailing_handlers.push(tokio::spawn(async move {
                                mailer.send(&utils::mail_message(
                                    &job,
                                    MailReason::MaxWaitElapsed,
                                    &smtp_configs,
                                ))
                            }));
                        }
                        warn!("Job {} elapsed max waiting time", job);
                        break;
                    }
                }

                todo!()
            }
            Err(job) => {
                if let Some(ref mailer) = mail_relay {
                    let message = match verified_jobs.get(&job) {
                        Some(context) => {
                            let reason = if context.0 {
                                MailReason::ErrorToPlay
                            } else {
                                MailReason::ErrorToCancel
                            };
                            utils::mail_message(&job, reason, &smtp_configs)
                        }
                        None => todo!(),
                    };

                    match mailer.send(&message) {
                        Ok(res) => {
                            debug!("Sent mail for job {}: {}", job, res.code());
                        }
                        Err(error) => error!(
                            "Fail to send a email for job {}: {}\n{:?}",
                            job, error, message
                        ),
                    };
                } else {
                    error!("Fail to cancel job {job}");
                }
            }
        }
    }

    for handle in mailing_handlers {
        let _ = handle.await;
    }
}
