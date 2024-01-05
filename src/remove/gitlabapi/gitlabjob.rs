use crate::gitlabapi::prelude::*;

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

    /// Get a tuple from an option with serde_json::Value and number of pages as u64
    async fn get_json(&self, url: &String) -> Option<(Value, u64)> {
        let resp = self.api_get(url);
        debug!("Getting json from: {url}");

        match resp.send().await {
            Err(e) => {
                error!("Error while getting {url}: {e}");
                None
            }
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
        }
    }

    /// Post facilitator with serde_json::Value as post body
    async fn post_json(&self, url: String, json: Value) -> Option<Value> {
        let resp = self.api_post(url.as_str(), json);

        match resp.send().await {
            Err(e) => {
                error!("Error while posting to {url}: {e}");
                None
            }
            Ok(response) => {
                debug!("HTTP Response Headers: {:?}", response.headers());
                debug!("HTTP Response Status: {:?}", response.status());
                debug!("HTTP Response Url: {:?}", response.url());
                match response.text().await {
                    Err(_) => None,
                    Ok(text) => Self::parse_json(text),
                }
            }
        }
    }

    /// Get projects ids from a Gitlab group
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

        loop {
            let new_uri = format!("{}&page={}", &base_uri, current_page);
            let num_pages;

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

            if current_page >= num_pages {
                break;
            }
            current_page += 1;
        }

        vec_projs
    }

    /// Get scoped jobs ids from a Gitlab project.
    pub async fn get_proj_jobs(&self, projid: u64, scope: JobScope) -> Vec<u64> {
        let uri = format!(
            "/api/v4/projects/{}/jobs?per_page=100&order_by=id&sort=asc&scope={}",
            projid, scope
        );
        let mut current_page = 1;
        let mut map_jobs: Vec<u64> = vec![];

        let mut new_uri;
        let mut num_pages;

        loop {
            new_uri = format!("{}&page={}", uri, current_page);

            let parse_json = self.get_json(&new_uri).await;

            // map_jobs.insert(project, vec![]);
            if let Some((json, pages)) = parse_json {
                num_pages = pages;
                match json.as_array() {
                    Some(vec_json) => {
                        vec_json.iter().for_each(|proj| {
                            if let Some(val) = proj["id"].as_u64() {
                                map_jobs.push(val)
                            } else {
                                error!("Unable to get jobs for project {projid}");
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

            if current_page >= num_pages {
                break;
            }
            current_page += 1;
        }

        map_jobs
    }

    /// Recover trigger variables from a Gitlab pipeline.
    async fn get_pipe_vars(&self, projid: u64, pipelineid: u64) -> HashMap<String, String> {
        let uri = format!("/api/v4/projects/{projid}/pipelines/{pipelineid}/variables");

        let mut hashmap_out: HashMap<String, String> = HashMap::new();

        let mut new_uri;
        let mut current_page = 1;

        loop {
            new_uri = format!("{}?per_page=100&page={}", &uri, current_page);
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

            if current_page >= num_pages {
                break;
            }
            current_page += 1;
        }

        hashmap_out
    }

    /// Get some informations from a project. *namely the project "name"*.
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

    /// The actual getter of job's informations, returning as `Jobinfo` struct.
    pub async fn get_jobinfo(&self, projid: u64, jobid: u64) -> Option<JobInfo> {
        let uri = format!("/api/v4/projects/{projid}/jobs/{jobid}");

        let (parse_json, project_infos) = join!(self.get_json(&uri), self.get_proj_info(projid));

        let mut jobinfo = JobInfo {
            id: Some(jobid),
            proj_id: Some(projid),
            ..Default::default()
        };
        // jobinfo.proj_id = Some(projid);
        // jobinfo.id = Some(jobid);

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

                jobinfo.source_id = variables.get("source_id").map(|v| v.parse().unwrap_or(0));
            };

            return Some(jobinfo);
        };

        None
    }

    /// Scans the configured project and project group for its children.
    async fn get_inner_projs(&self) -> HashSet<u64> {
        let mut vec_out = HashSet::new();

        if let Some(lone_proj) = self.config.project_id {
            vec_out.insert(lone_proj);
        }

        if self.config.group_id.is_some() {
            self.get_group_projs().await.iter().for_each(|proj| {
                vec_out.insert(*proj);
            });
        }

        vec_out
    }

    /// Scans scoped jobs orderning by project ids.
    pub async fn get_jobs_by_project(&self, scope: JobScope) -> HashMap<u64, Vec<JobInfo>> {
        let projects = self.get_inner_projs().await;

        let stream_projects = stream::iter(&projects)
            .map(|proj| async move { (proj, self.get_proj_jobs(*proj, scope).await) })
            .buffer_unordered(STREAM_BUFF_SIZE)
            .fuse();
        tokio::pin!(stream_projects);

        let mut projid_jobid_tuple: Vec<(u64, u64)> = vec![];
        while let Some((proj, mut jobs)) = stream_projects.next().await {
            jobs.sort();
            jobs.reverse();
            jobs.iter().for_each(|jobid| {
                projid_jobid_tuple.push((*proj, *jobid));
            });
        }

        let mut stream_jobs = stream::iter(&projid_jobid_tuple)
            .map(|(projid, jobid)| async move { (projid, self.get_jobinfo(*projid, *jobid).await) })
            .buffer_unordered(STREAM_BUFF_SIZE)
            .fuse();

        let mut proj_jobs: HashMap<u64, Vec<JobInfo>> = HashMap::new();
        while let Some((projid, jobinfo)) = stream_jobs.next().await {
            if let Some(jobinfo) = jobinfo {
                proj_jobs
                    .entry(*projid)
                    .and_modify(|jobs| {
                        jobs.push(jobinfo.clone());
                    })
                    .or_insert(Vec::from([jobinfo]));
            }
        }

        proj_jobs
    }

    /// Scans scoped jobs orderning by project ids and its pipelines.
    pub async fn get_jobs_by_proj_and_pipeline(
        &self,
        scope: JobScope,
    ) -> HashMap<u64, HashMap<u64, Vec<JobInfo>>> {
        let proj_jobs = self.get_jobs_by_project(scope).await;

        let mut output: HashMap<u64, HashMap<u64, Vec<JobInfo>>> = HashMap::new();

        proj_jobs.iter().for_each(|(projid, jobs)| {
            jobs.iter().for_each(|jobinfo| {
                let pipeid = jobinfo.pipeline_id.unwrap();
                output
                    .entry(*projid)
                    .and_modify(|pipe_hash| {
                        pipe_hash
                            .entry(pipeid)
                            .and_modify(|jobs_vec| {
                                jobs_vec.push(jobinfo.to_owned());
                            })
                            .or_insert(vec![jobinfo.to_owned()]);
                    })
                    .or_insert(HashMap::from([(pipeid, vec![jobinfo.to_owned()])]));
            });
        });

        output
    }

    /// Scans for all scoped jobs from configured group and project.
    pub async fn get_all_jobs(&self, scope: JobScope) -> HashSet<JobInfo> {
        let mut vec_out: HashSet<JobInfo> = HashSet::new();

        self.get_jobs_by_project(scope)
            .await
            .iter()
            .for_each(|(_, jobs)| {
                jobs.iter().for_each(|jobinfo| {
                    vec_out.insert(jobinfo.to_owned());
                });
            });

        vec_out
    }

    /// Starts a manual (paused) job.
    pub async fn play_job(&self, job: &JobInfo) -> Result<(), String> {
        let url = format!(
            "api/v4/projects/{}/jobs/{}/play",
            job.proj_id.unwrap(),
            job.id.unwrap()
        );

        // let form = HashMap::from([("", "")]);

        let resp = self.post_json(url, Value::String("".to_owned()));

        if let Some(response) = resp.await {
            debug!("Job played: {:?}", response);
            return Ok(());
        }

        Err(format!("Couldn't play job {:?}", job))
    }

    /// Cancels a job.
    pub async fn cancel_job(&self, job: &JobInfo) -> Result<(), String> {
        let url = format!(
            "api/v4/projects/{}/jobs/{}/cancel",
            job.proj_id.unwrap(),
            job.id.unwrap()
        );

        // let form = HashMap::from([("", "")]);

        let resp = self.post_json(url, Value::String("".to_owned()));

        if let Some(response) = resp.await {
            debug!("Job canceled: {:?}", response);
            return Ok(());
        }

        Err(format!("Couldn't cancel job {:?}", job))
    }

    /// Get an updated job status as `JobScope` enum.
    pub async fn get_new_job_status(&self, job: &JobInfo) -> Option<JobScope> {
        let projid = job.proj_id.unwrap();
        let jobid = job.id.unwrap();

        let uri = format!("/api/v4/projects/{projid}/jobs/{jobid}");

        if let Some((json, _)) = self.get_json(&uri).await {
            return json["status"]
                .as_str()
                .map(|v| JobScope::from(v.to_owned()));
        }

        None
    }

    /// Inspect a project for its git tags.
    pub async fn get_proj_git_tags(&self, projid: u64) -> Vec<String> {
        let url = format!("api/v4/projects/{projid}/repository/tags?order_by=updated");

        let mut got_tags = vec![];

        if let Some((resp, _)) = self.get_json(&url).await {
            if let Some(tags_list) = resp.as_array() {
                tags_list.iter().for_each(|tag| {
                    if let Some(tag_name) = tag["name"].as_str() {
                        got_tags.push(tag_name.to_owned())
                    }
                })
            }
        }

        got_tags
    }

    pub async fn bulk_jobs_cancel<'a, 'b>(
        &'a self,
        jobs: &HashSet<&'b JobInfo>,
    ) -> Result<(), HashSet<&'b JobInfo>> {
        let stream = stream::iter(jobs)
            .map(|job| async {
                match self.cancel_job(&(**job).clone()).await {
                    Ok(_) => None,
                    Err(_) => Some(*job),
                }
            })
            .buffer_unordered(STREAM_BUFF_SIZE)
            .collect::<HashSet<Option<&JobInfo>>>()
            .await;

        if stream.is_empty() {
            Ok(())
        } else {
            let mut error_jobs: HashSet<&JobInfo> = HashSet::new();
            stream.iter().for_each(|job| {
                if let Some(job) = job {
                    error_jobs.insert(job);
                }
            });
            Err(error_jobs)
        }
    }
}

// Tests for private methods
#[cfg(test)]
mod test_gitlabjob {
    use super::*;

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

        let api = GitlabJOB::new(&config);

        let response = api.get_proj_info(config.project_id.unwrap()).await;

        debug!("Project infos: {:?}", response);
    }

    #[tokio::test]
    #[ignore = "specific pipeline"]
    async fn test_get_pipe_vars() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(&config);

        let specify_project = 513_u64;
        let specify_pipeline = 15253_u64;

        let pipe_vars = api.get_pipe_vars(specify_project, specify_pipeline).await;

        debug!("HashMap from pipeline variables: {:?}", pipe_vars);
    }
    #[tokio::test]
    #[ignore = "It only triggers a pipeline"]
    async fn create_job() {
        init();

        use std::env;

        let token_trigger = env::var("TOKEN_TRIGGER").unwrap_or("123456".to_owned());
        let destination_test = env::var("TEST_DESTINATION").unwrap_or("test@test.tst".to_owned());

        let post_body = serde_json::json!({
            // "ref":"master", 
            "ref":"main", 
            "token": token_trigger,
            "variables":{
                "trigger_email": destination_test,
                "source_id":"306",
                "ref_source":"main",
                "PROD_TAG":"PROD-0.0.1"}});

        let config = Config::load_config().unwrap();
        let projid = config.project_id.unwrap();

        let api = GitlabJOB::new(&config);

        let output = api
            .post_json(
                format!("api/v4/projects/{projid}/trigger/pipeline"),
                post_body,
            )
            .await;

        debug!("Response: {:?}", output);
    }
    #[tokio::test]
    async fn test_get_inner_projects() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(&config);

        let output = api.get_inner_projs().await;

        let mut sorted: Vec<u64> = output.into_iter().collect();
        sorted.sort();

        debug!("{:?}", sorted);
        debug!("Total projects: {}", sorted.len());
    }
}
