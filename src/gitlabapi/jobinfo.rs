use std::convert::From;
use std::fmt::Display;

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
            JobScope::Invalid => write!(f, "invalid"),
        }
    }
}

impl From<String> for JobScope {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "created" => JobScope::Created,
            "pending" => JobScope::Pending,
            "running" => JobScope::Running,
            "failed" => JobScope::Failed,
            "success" => JobScope::Success,
            "canceled" => JobScope::Canceled,
            "skipped" => JobScope::Skipped,
            "waiting_for_resource" => JobScope::WaitingForResource,
            "manual" => JobScope::Manual,
            _ => JobScope::Invalid,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct JobInfo {
    pub id: Option<u64>,
    pub status: Option<JobScope>,
    pub url: Option<String>,
    pub proj_name: Option<String>,
    pub proj_id: Option<u64>,
    pub pipeline_id: Option<u64>,
    pub source_id: Option<u64>,
    pub user_mail: Option<String>,
    pub branch: Option<String>,
    pub git_tag: Option<String>,
}

impl JobInfo {
    pub fn default() -> Self {
        Self {
            proj_name: None,
            proj_id: None,
            source_id: None,
            branch: None,
            pipeline_id: None,
            user_mail: None,
            git_tag: None,
            url: None,
            id: None,
            status: None,
        }
    }

    pub fn to_html(&self) -> String {
        let default_string = "unknown".to_owned();

        let proj_name = self.proj_name.as_ref().unwrap_or(&default_string);
        let proj_id = self.proj_id.unwrap_or(0);
        let source_id = self.source_id.unwrap_or(0);
        let branch = self.branch.as_ref().unwrap_or(&default_string);
        let pipeline_id = self.pipeline_id.unwrap_or(0);
        let user_mail = self.user_mail.as_ref().unwrap_or(&default_string);
        let git_tag = self.git_tag.as_ref().unwrap_or(&default_string);
        let url = self.url.as_ref().unwrap_or(&default_string);
        let job_id = self.id.unwrap_or(0);
        let status = self.status.unwrap_or(JobScope::Invalid);


        format!(
            r#"
            <ul>
                <li>Project name: <b>{proj_name}</b></li>
                <li>Git tag: <b>{git_tag}</b></li>
                <li>Branch: <b>{branch}</b></li>
                <li>Source project id: <b>{source_id}</b></li>
                <li>Deploy project id: <b>{proj_id}</b></li>
                <li>Deploy pipeline id: <b>{pipeline_id}</b></li>
                <li>User mail: <b>{user_mail}</b></li>
                <li>Job URL: <b>{url}</b></li>
                <li>Job id: <b>{job_id}</b></li>
                <li>Job status: <b>{status}</b></li>
            </ul>
            "#
        )
    }
}

impl Display for JobInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} from project {}", self.id.unwrap_or(00), self.proj_name.to_owned().unwrap_or("unknown".to_owned()))
    }
}
