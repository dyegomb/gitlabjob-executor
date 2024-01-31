// use async_trait::async_trait;

mod getters;
mod getters_traits;
mod jobinfo;
pub mod setters;
mod tests;
mod utils;

pub use configloader::Config;

/// Specify how many concurrent tasks
pub const STREAM_BUFF_SIZE: usize = 15;

pub mod prelude {
    pub use super::getters_traits::*;
    pub use super::jobinfo::{JobInfo, JobScope};
    pub use super::setters;
    pub use super::setters::JobActions;
    pub use super::Config;
    pub use super::GitlabJOB;
    pub use super::STREAM_BUFF_SIZE;
    pub use super::{GroupID, JobID, PipelineID, ProjectID};
    pub use log::{debug, error, warn};
    pub use serde_json::Value;
}

/// API caller configured from `Config` module.
pub struct GitlabJOB {
    pub config: Config,
}

type ID = u64;

impl GitlabJOB {
    pub fn new(config: &Config) -> Self {
        GitlabJOB {
            config: config.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GroupID(pub ID);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectID(pub ID);
#[derive(Debug, Clone, Copy)]
pub struct JobID(pub ID);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PipelineID(pub ID);
