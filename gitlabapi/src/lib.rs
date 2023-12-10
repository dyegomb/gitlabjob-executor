mod getters;
mod utils;

pub use configloader::Config;

/// Specify how many concurrent tasks
pub const STREAM_BUFF_SIZE: usize = 15;

mod prelude {
    pub use super::Config;
    pub use super::GitlabJOB;
    pub use super::JobScope;
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

pub struct GroupID(u64);
pub struct ProjectID(u64);
pub struct JobID(u64);
pub struct PipelineID(u64);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum JobScope {
    Created,
    Pending,
    Running,
    Failed,
    Success,
    Canceled,
    Skipped,
    WaitingForResource,
    Manual,
    Invalid,
}
