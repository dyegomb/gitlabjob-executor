use std::fmt::Display;
use std::convert::From;

/// Jobs scopes: https://docs.gitlab.com/ee/api/jobs.html#list-project-jobs
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
}

impl Display for JobScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobScope::Created => write!(f, "created"),
            JobScope::Pending => write!(f, "pending"),
            JobScope::Running => write!(f, "running"),
            JobScope::Failed => write!(f, "failed"),
            JobScope::Success => write!(f, "success"),
            JobScope::Canceled => write!(f, "canceled"),
            JobScope::Skipped => write!(f, "skipped"),
            JobScope::WaitingForResource => write!(f, "waiting_for_resource"),
            JobScope::Manual => write!(f, "manual"),
        }
    }
}

impl From<String> for JobScope {
    fn from(value: String) -> Self {
        match value.as_str() {
            "created" => JobScope::Created,
            "pending" => JobScope::Pending,
            "running" => JobScope::Running,
            "failed" => JobScope::Failed,
            "success" => JobScope::Success,
            "canceled" => JobScope::Canceled,
            "skipped" => JobScope::Skipped,
            "waiting_for_resource" => JobScope::WaitingForResource,
            "manual" => JobScope::Manual,
            _ => JobScope::Skipped
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct JobInfo {
    pub id: Option<u64>,
    pub status: Option<JobScope>,
    pub url: Option<String>,
    pub proj_name: Option<String>,
    pub proj_id: Option<u64>,
    pub pipeline_id: Option<u64>,
    pub source_id: Option<String>,
    pub user_mail: Option<String>,
    pub branch: Option<String>,
    pub git_tag: Option<String>,
}


impl JobInfo {
    pub fn default() -> Self {
        Self {
            id: None,
            url: None,
            proj_name: None,
            pipeline_id: None,
            source_id: None,
            user_mail: None,
            branch: None,
            git_tag: None,
            status: None,
            proj_id: None,
        }
    }
}