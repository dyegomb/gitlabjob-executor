/// Jobs scopes: https://docs.gitlab.com/ee/api/jobs.html#list-project-jobs
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

pub struct JobInfo {
    // 'jobid': jobs_json.get("id"),
    // 'job_url': jobs_json.get("web_url"),
    // 'nome_projeto': proj_json.get("name"),
    // "pipelineid": pipid,
    // "source_id": source_id,
    // "user_mail": user_mail,
    // "branch": ref_source or "não informada",
    // "versao_tag": prod_tag or "não informada",
    pub id: Option<u64>,
    pub url: Option<String>,
    pub proj_name: Option<String>,
    pub pipeline_id: Option<u64>,
    pub source_id: Option<String>,
    pub user_mail: Option<String>,
    pub branch: Option<String>,
    pub git_tag: Option<String>,
}

impl JobInfo {
    pub fn new(
        id: Option<u64>,
        url: Option<String>,
        proj_name: Option<String>,
        pipeline_id: Option<u64>,
        source_id: Option<String>,
        user_mail: Option<String>,
        branch: Option<String>,
        git_tag: Option<String>,
    ) -> Self {
        Self {
            id,
            url,
            proj_name,
            pipeline_id,
            source_id,
            user_mail,
            branch,
            git_tag,
        }
    }
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
        }
    }
}