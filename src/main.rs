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

use std::collections::HashSet;
use tokio::time as tktime;

use configloader::prelude::*;
use gitlabapi::prelude::*;

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
    // Set default log level for INFO, changed with "RUST_LOG"
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config = match Config::load_config() {
        Ok(conf) => conf,
        Err(err) => panic!("Error loading configurations. {}", err),
    };

    let mail_relay = utils::mailrelay_buid(&config).await;

    // Scan projects for Manual jobs
    let api = GitlabJOB::new(&config);
    let mut playable_jobs: HashSet<&JobInfo> = HashSet::new();
    let mut cancel_jobs: HashSet<&JobInfo> = HashSet::new();
    let mut mail_jobs_list: Vec<(&JobInfo, MailReason)> = vec![];

    let jobs = match config.group_id {
        Some(group_id) => api.get_jobs(GroupID(group_id), JobScope::Manual).await,
        None => match config.project_id {
            Some(proj_id) => api.get_jobs(ProjectID(proj_id), JobScope::Manual).await,
            None => panic!("There's no project to scan for jobs."),
        },
    };

    log::info!(
        "Projects with {} status jobs: {:?}",
        JobScope::Manual,
        jobs.keys()
    );
}
