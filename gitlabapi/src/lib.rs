// use async_trait::async_trait;

mod getters;
mod getters_traits;
mod jobinfo;
mod tests;
mod utils;

pub use configloader::Config;

/// Specify how many concurrent tasks
pub const STREAM_BUFF_SIZE: usize = 15;

pub mod prelude {
    pub use super::getters_traits::*;
    pub use super::jobinfo::{JobInfo, JobScope};
    pub use super::Config;
    pub use super::GitlabJOB;
    pub use super::STREAM_BUFF_SIZE;
    pub use super::{GroupID, JobID, PipelineID, ProjectID};
    pub use log::{debug, warn};
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
pub struct GroupID(pub u64);
#[derive(Debug, Clone, Copy)]
pub struct ProjectID(pub u64);
#[derive(Debug, Clone, Copy)]
pub struct JobID(pub u64);
#[derive(Debug, Clone, Copy)]
pub struct PipelineID(pub u64);
