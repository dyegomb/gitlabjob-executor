// use async_trait::async_trait;

mod getters;
mod getters_traits;
mod jobinfo;
mod utils;

pub use configloader::Config;

/// Specify how many concurrent tasks
pub const STREAM_BUFF_SIZE: usize = 15;

mod prelude {
    pub use super::STREAM_BUFF_SIZE;
    pub use super::Config;
    pub use super::GitlabJOB;
    pub use super::jobinfo::{JobInfo, JobScope};
    pub use super::{GroupID, JobID, PipelineID, ProjectID};
    pub use log::{debug, error, warn};
    pub use serde_json::Value;
}

/// API caller configured from `Config` module.
pub struct GitlabJOB {
    pub config: Config,
}

impl GitlabJOB {
    pub fn new(config: &Config) -> Self {
        GitlabJOB {
            config: config.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GroupID(u64);
#[derive(Debug, Clone, Copy)]
pub struct ProjectID(u64);
#[derive(Debug, Clone, Copy)]
pub struct JobID(u64);
#[derive(Debug, Clone, Copy)]
pub struct PipelineID(u64);
