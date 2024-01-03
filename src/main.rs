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
use log::error;
use std::collections::{HashMap, HashSet};
use tokio::time as tktime;
// use tokio_stream::StreamExt;

use configloader::prelude::*;
use gitlabapi::{prelude::*, setters::JobActions};

mod tests;
mod utils;

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
    // let mut playable_jobs: HashSet<&JobInfo> = HashSet::new();
    // let mut cancel_jobs: HashSet<&JobInfo> = HashSet::new();
    let mut mail_jobs_list: Vec<(Option<&JobInfo>, MailReason)> = vec![];

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

    // let pipelines_tocancel = utils::pipelines_tocancel(&proj_jobs);

    // Classify jobs
    // for (project, jobs) in &proj_jobs {
    //     for job in jobs {
    //         match job.pipeline_id {
    //             Some(pipe_id) => {
    //                 match pipelines_tocancel
    //                     .get(project)
    //                     .unwrap()
    //                     .contains(&PipelineID(pipe_id))
    //                 {
    //                     true => {
    //                         cancel_jobs.insert(job);
    //                     }
    //                     false => {
    //                         if job.git_tag.is_some() && job.source_id.is_some() {
    //                             // playable_jobs.insert(job);
    //                             let tags = api.get_tags(ProjectID(job.source_id.unwrap())).await;
    //                         } else {
    //                             playable_jobs.insert(job);
    //                         }
    //                     }
    //                 };
    //             }
    //             None => {
    //                 warn!("A job without pipeline {}", job);
    //                 cancel_jobs.insert(job);
    //             }
    //         }
    //     }
    // }

    // // Cancel jobs
    // let stream_cancel = stream::iter(cancel_jobs)
    //     .map(|job| api.cancel_job(job))
    //     .buffer_unordered(STREAM_BUFF_SIZE)
    //     .fuse();
    // tokio::pin!(stream_cancel);

    // while let Some(job_result) = stream_cancel.next().await {
    //     match job_result {
    //         Ok(job) => {
    //             mail_jobs_list.push((Some(job), MailReason::Duplicated));
    //         }
    //         Err(e) => {
    //             error!("Error to cancel job: {}", e);
    //             mail_jobs_list.push((None, MailReason::ErrorToCancel))
    //         }
    //     }
    // }

    // // Play jobs
    // let stream_play = stream::iter(playable_jobs)
    //     .map(|job| api.play_job(job))
    //     .buffer_unordered(STREAM_BUFF_SIZE)
    //     .fuse();
    // tokio::pin!(stream_play);

    // while let Some(job_result) = stream_play.next().await {}
    let verified_jobs = utils::validate_jobs(&api, &proj_jobs).await;

    let mail_relay = match mail_relay_handle.await {
        Ok(relay) => relay,
        Err(e) => {
            warn!("No email will be sent. {}", e);
            None
        }
    };
}
