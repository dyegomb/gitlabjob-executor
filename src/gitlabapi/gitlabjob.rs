use crate::gitlabapi::prelude::*;

pub struct GitlabJOB {
    pub config: Config,
}

impl GitlabJOB {
    pub fn new(config: Config) -> Self {
        GitlabJOB { config }
    }

    /// Get a tuple from an option with serde_json::Value and number of pages as u64
    async fn get_json(&self, url: &String) -> Option<(Value, u64)> {
        let resp = self.api_get(url).send().await;

        match resp {
            Ok(response) => {
                let headers = response.headers().clone();
                match response.text().await {
                    Ok(text) => {
                        let num_pages: u64 = match headers.get("x-total-pages") {
                            Some(total_pages) => {
                                if let Ok(num_str) = total_pages.to_str() {
                                    num_str.parse().unwrap_or(1)
                                } else {
                                    1
                                }
                            }
                            None => 1,
                        };

                        // debug!("Path \"{url}\" gave json:\n{text}");
                        Self::parse_json(text).map(|val| (val, num_pages))
                    }
                    Err(_) => None,
                }
            }
            Err(e) => {
                error!("Error while calling {url}: {e}");
                None
            }
        }
    }

    pub async fn get_group_projs(&self) -> Vec<u64> {
        if self.config.group_id.is_none() {
            return vec![];
        };

        let base_uri = format!(
            "/api/v4/groups/{}/projects?pagination=keyset&simple=true&per_page=100&order_by=id&sort=asc",
            self.config.group_id.unwrap()
        );

        let mut vec_projs: Vec<u64> = vec![];

        let mut current_page = 1;
        let mut num_pages;

        loop {
            let new_uri = format!("{}&page={}", &base_uri, current_page);

            if let Some((json, total_pages)) = self.get_json(&new_uri).await {
                num_pages = total_pages;
                if let Some(vec_json) = json.as_array() {
                    vec_json.iter().for_each(|proj| {
                        if let Some(val) = proj["id"].as_u64() {
                            vec_projs.push(val);
                        }
                    });
                }
            } else {
                num_pages = 1;
            }

            if current_page >= num_pages { break; }
            current_page += 1;
        }

        vec_projs
    }

    pub async fn get_proj_jobs(&self, project: u64, scope: JobScope) -> HashMap<u64, Vec<u64>> {
        let uri = format!(
            "/api/v4/projects/{}/jobs?pagination=keyset&per_page=100&order_by=id&sort=asc&scope={}",
            project, scope
        );
        let mut current_page = 1;
        let mut map_jobs: HashMap<u64, Vec<u64>> = HashMap::new();

        let mut new_uri;
        let mut num_pages;

        loop {
            new_uri = format!("{}&page={}", uri, current_page);

            let parse_json = self.get_json(&new_uri).await;

            map_jobs.insert(project, vec![]);
            if let Some((json, pages)) = parse_json {
                num_pages = pages;
                match json.as_array() {
                    Some(vec_json) => {
                        vec_json.iter().for_each(|proj| {
                            let val = proj["id"].as_u64().unwrap();
                            if let Some(proj) = map_jobs.get_mut(&project) {
                                proj.push(val);
                            } else {
                                error!("Unable to fill job {val} for project {project}");
                            }
                        });
                    }
                    None => {
                        warn!("No jobs found in {}", uri);
                    }
                }
            } else {
                num_pages = 1;
            };

            if current_page >= num_pages { break; }
            current_page += 1;
        }

        map_jobs
    }

    async fn get_pipe_vars(&self, projid: u64, pipelineid: u64) -> HashMap<String, String> {
        let uri = format!("/api/v4/projects/{projid}/pipelines/{pipelineid}/variables");

        let mut hashmap_out: HashMap<String, String> = HashMap::new();

        let mut new_uri;
        let mut current_page = 1;

        loop {
            new_uri = format!(
                "{}?pagination=keyset&per_page=100&page={}",
                &uri, current_page
            );
            let num_pages;

            if let Some((vars_obj, pages)) = self.get_json(&new_uri).await {
                num_pages = pages;
                if let Some(vec_vars) = vars_obj.as_array() {
                    vec_vars.iter().for_each(|var| {
                        if let Some(key) = var["key"].as_str() {
                            if let Some(value) = var["value"].as_str() {
                                hashmap_out.insert(key.to_owned(), value.to_owned());
                            }
                        }
                    });
                }
            } else {
                num_pages = 1;
            };

            if current_page >= num_pages { break; }
            current_page += 1;
        }

        hashmap_out
    }

    async fn get_proj_info(&self, projid: u64) -> HashMap<String, String> {
        let uri = format!("/api/v4/projects/{projid}");

        let parse_json = self.get_json(&uri).await;
        let mut hash_map: HashMap<String, String> = HashMap::new();
        if let Some((json, _)) = parse_json {
            if let Some(name) = json["name"].as_str() {
                hash_map.insert("name".to_owned(), name.to_owned());
            }
        }
        hash_map
    }

    pub async fn get_jobinfo(&self, projid: u64, jobid: u64) -> Option<JobInfo> {
        let uri = format!("/api/v4/projects/{projid}/jobs/{jobid}");

        let (parse_json, project_infos) = join!(self.get_json(&uri), self.get_proj_info(projid));

        let mut jobinfo = JobInfo::default();
        jobinfo.proj_id = Some(projid);
        jobinfo.id = Some(jobid);

        if let Some((json, _)) = parse_json {

            jobinfo.status = json["status"]
                .as_str()
                .map(|v| JobScope::from(v.to_owned()));

            jobinfo.url = json["web_url"].as_str().map(|v| v.to_owned());

            if let Some(proj_name) = project_infos.get("name") {
                jobinfo.proj_name = Some(proj_name.to_owned());
            };

            if let Some(pipe_info) = json["pipeline"].as_object() {
                let variables;
                if let Some(pipe_id) = pipe_info["id"].as_u64() {
                    jobinfo.pipeline_id = Some(pipe_id);
                    variables = self.get_pipe_vars(projid, pipe_id).await;
                } else {
                    variables = HashMap::new();
                }

                jobinfo.user_mail = match variables.get("trigger_email") {
                    Some(mail) => Some(mail.to_owned()),
                    None => match json["commit"].as_object() {
                        Some(commit_obj) => commit_obj["committer_email"]
                            .as_str()
                            .map(|email| email.to_owned()),
                        None => None,
                    },
                };

                if let Some(prod_tag_key) = &self.config.production_tag_key {
                    jobinfo.git_tag = variables.get(prod_tag_key).cloned();
                } else {
                    jobinfo.git_tag = match json["commit"].as_object() {
                        Some(commit_obj) => {
                            commit_obj.get("ref_name").map(|tag| match tag.as_str() {
                                Some(tag) => tag.to_owned(),
                                None => "".to_owned(),
                            })
                        }
                        None => None,
                    }
                };

                jobinfo.branch = match variables.get("ref_source") {
                    Some(from_trigger) => Some(from_trigger.to_owned()),
                    None => json["ref"].as_str().map(|ref_branch| ref_branch.to_owned()),
                };

                jobinfo.source_id = variables.get("source_id").map(|v| v.to_owned());
            };

            return Some(jobinfo);
        };

        None
    }

    pub async fn get_all_jobs(&self, scope: JobScope) -> HashSet<JobInfo> {
        let mut projs_scan_list: Vec<u64> = vec![];

        if let Some(lone_proj) = self.config.project_id {
            projs_scan_list.push(lone_proj)
        }

        if self.config.group_id.is_some() {
            self.get_group_projs()
                .await
                .iter()
                .for_each(|proj| projs_scan_list.push(*proj))
        }

        let mut proj_stream = tokio_stream::iter(&projs_scan_list);
        let mut jobs_list: Vec<(u64, u64)> = vec![];

        // Scan scoped jobs
        while let Some(proj) = proj_stream.next().await {
            debug!("Searching for jobs in project {}", proj);
            self.get_proj_jobs(*proj, scope).await
                .iter()
                .for_each(|(proj_id, jobs)| {
                    jobs.iter()
                        .for_each(|job_id| {
                            jobs_list.push((*proj_id, *job_id))
                        });
                });
        }

        // Get jobs info
        let mut vec_out: HashSet<JobInfo> = HashSet::new();

        let mut jobs_stream = tokio_stream::iter(&jobs_list);

        while let Some((projid, jobid)) = jobs_stream.next().await {
            if let Some(job_info) = self.get_jobinfo(*projid, *jobid).await {
                vec_out.insert(job_info);
            }
        };

        vec_out
    }
}

#[cfg(test)]
mod test_gitlabjob {
    use super::*;

    use crate::load_config;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[tokio::test]
    async fn test_get_proj_info() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let response = api.get_proj_info(config.project_id.unwrap()).await;

        debug!("Project infos: {:?}", response);
    }

    #[tokio::test]
    #[ignore = "specific pipeline"]
    async fn test_get_pipe_vars() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let specify_project = 513_u64;
        let specify_pipeline = 15253_u64;

        let pipe_vars = api.get_pipe_vars(specify_project, specify_pipeline).await;

        debug!("HashMap from pipeline variables: {:?}", pipe_vars);
    }
}