/// Jobs scopes: https://docs.gitlab.com/ee/api/jobs.html#list-project-jobs
#[derive(Debug)]
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

#[derive(Debug)]
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
    pub status: Option<JobScope>,
    pub url: Option<String>,
    pub proj_name: Option<String>,
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
        }
    }
}