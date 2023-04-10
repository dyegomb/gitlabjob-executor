//https://docs.gitlab.com/ee/api/rest/index.html
use crate::gitlabapi::jobinfo::{JobInfo, JobScope};
use crate::load_config::Config;

use futures::join;
use log::{debug, error, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::collections::HashMap;

mod jobinfo;
mod mod_tests;

pub struct GitlabJOB {
    config: Config,
}

impl GitlabJOB {
    pub fn new(config: Config) -> Self {
        GitlabJOB { config }
    }

    fn parse_json(text: String) -> Option<Value> {
        if let Ok(parsed_json) = serde_json::from_str::<Value>(&text) {
            Some(parsed_json)
        } else {
            error!("Error while parsing to json from: \n{}", text);
            None
        }
    }

    fn api_builder(&self) -> reqwest::ClientBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            "PRIVATE-TOKEN",
            HeaderValue::from_str(self.config.private_token.as_ref().unwrap()).unwrap(),
        );
        headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());

        reqwest::ClientBuilder::new().default_headers(headers)
    }

    fn gen_url(&self, path: &str) -> reqwest::Url {
        let new_uri = self.config.base_url.clone().unwrap() + path;

        match reqwest::Url::parse(&new_uri) {
            Ok(url) => url,
            Err(error) => {
                error!("Error while parsing url: {}", new_uri);
                panic!("Error while parsing url \"{}\": {}", new_uri, error)
            }
        }
    }

    fn api_get(&self, url: &String) -> reqwest::RequestBuilder {
        let uri = self.gen_url(url);

        debug!("Building request for {uri}");

        match self.api_builder().build() {
            Ok(getter) => getter.get(uri),
            Err(err) => {
                panic!("Coudn't construct the api caller: {}", err);
            }
        }
    }

    async fn get_json(&self, url: &String) -> Option<Value> {
        let resp = self.api_get(url).send().await;

        match resp {
            Ok(response) => match response.text().await {
                Ok(text) => {
                    debug!("Path \"{url}\" gave json:\n{text}");
                    Self::parse_json(text)
                }
                Err(_) => None,
            },
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

        let uri = format!(
            "/api/v4/groups/{}/projects?pagination=keyset&per_page=100&order_by=id&sort=asc",
            self.config.group_id.unwrap()
        );

        let parse_json = self.get_json(&uri).await;

        let mut vec_projs: Vec<u64> = vec![];

        if let Some(json) = parse_json {
            match json.as_array() {
                Some(vec_json) => {
                    vec_json.iter().for_each(|proj| {
                        let val = proj["id"].as_u64().unwrap();
                        vec_projs.push(val);
                    });
                }
                None => {
                    debug!("No jobs found for {}", uri);
                    return vec![];
                }
            }
        }

        vec_projs
    }

    pub async fn get_proj_jobs(&self, project: u64, scope: JobScope) -> HashMap<u64, Vec<u64>> {
        let uri = format!(
            "/api/v4/projects/{}/jobs?per_page=100&order_by=id&sort=asc&scope={}",
            project, scope
        );

        let parse_json = self.get_json(&uri).await;

        let mut map_jobs: HashMap<u64, Vec<u64>> = HashMap::new();
        map_jobs.insert(project, vec![]);

        if let Some(json) = parse_json {
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
        };

        map_jobs
    }

    async fn get_pipe_vars(&self, projid: u64, pipelineid: u64) -> HashMap<String, String> {
        let uri = format!("/api/v4/projects/{projid}/pipelines/{pipelineid}/variables");

        let mut hashmap_out: HashMap<String, String> = HashMap::new();

        if let Some(vars_obj) = self.get_json(&uri).await {
            if let Some(vec_vars) = vars_obj.as_array() {
                vec_vars.iter().for_each(|var| {
                    if let Some(key) = var["key"].as_str() {
                        if let Some(value) = var["value"].as_str() {
                            hashmap_out.insert(key.to_owned(), value.to_owned());
                        }
                    }
                });
            }
        };

        hashmap_out
    }

    async fn get_proj_info(&self, projid: u64) -> HashMap<String, String> {
        let uri = format!("/api/v4/projects/{projid}");

        let parse_json = self.get_json(&uri).await;

        let mut hash_map: HashMap<String, String> = HashMap::new();
        if let Some(json) = parse_json {
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

        if let Some(json) = parse_json {
            jobinfo.id = json["id"].as_u64();
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
            };

            return Some(jobinfo);
        };

        None
    }
}
