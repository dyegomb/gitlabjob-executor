//https://docs.gitlab.com/ee/api/rest/index.html
use crate::gitlabapi::jobinfo::{JobInfo, JobScope};
use crate::load_config::Config;

// use env_logger::fmt;
use log::{debug, error, info, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use std::fmt::Display;

mod jobinfo;

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

pub struct GitlabJOB {
    config: Config,
}

impl GitlabJOB {
    pub fn new(config: Config) -> Self {
        GitlabJOB { config: config }
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

    pub async fn get_group_projs(&self) -> Vec<u64> {
        if self.config.group_id.is_none() {
            return vec![];
        };

        let uri = format!(
            "/api/v4/groups/{}/projects?pagination=keyset&per_page=100&order_by=id&sort=asc",
            self.config.group_id.unwrap()
        );

        let resp = self.api_get(&uri).send().await.unwrap().text().await;

        let parse_json;

        if let Ok(got_resp) = resp {
            parse_json = serde_json::from_str::<Value>(&got_resp);
        } else {
            panic!("Error parsing json response from {}", &uri);
        };

        let mut vec_projs: Vec<u64> = vec![];

        if let Ok(json) = parse_json {
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

    pub async fn get_proj_jobs(&self, project: u64, scope: JobScope) -> Vec<u64> {
        let uri = format!(
            "/api/v4/projects/{}/jobs?per_page=100&order_by=id&sort=asc&scope={}",
            project, scope
        );

        let resp = self.api_get(&uri).send().await.unwrap().text().await;

        let parse_json;
        if let Ok(got_resp) = resp {
            parse_json = serde_json::from_str::<Value>(&got_resp);
        } else {
            panic!("Error parsing json response from {}", &uri);
        };

        let mut vec_jobs: Vec<u64> = vec![];

        if let Ok(json) = parse_json {
            match json.as_array() {
                Some(vec_json) => {
                    vec_json.iter().for_each(|proj| {
                        let val = proj["id"].as_u64().unwrap();
                        vec_jobs.push(val);
                    });
                }
                None => {
                    debug!("No jobs found for {}", uri);
                    return vec![];
                }
            }
        };

        vec_jobs
    }

    pub fn get_jobinfo(&self, jobid: u64) -> JobInfo {
        todo!()
    }
}

#[cfg(test)]
mod test_http {
    use std::io::Write;

    use crate::load_config;

    use super::*;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    // #[test]
    #[tokio::test]
    async fn test_api_get() {
        init();
        let config = load_config().unwrap();

        let api = GitlabJOB::new(config);

        let response = api
            .api_get(&"/api/v4/projects".to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await;

        // debug!("{}", response.as_ref().unwrap());

        let parsed_json: Value = serde_json::from_str(response.as_ref().unwrap()).unwrap();

        parsed_json.as_array().iter().for_each(|projects| {
            projects.iter().for_each(|proj| {
                debug!("Project ID: {}", proj["id"]);
                debug!("Project links: {}", proj["_links"]);
            });
        });

        let _ = std::fs::File::create("/tmp/test.json")
            .unwrap()
            .write(response.unwrap().as_bytes());
    }

    #[tokio::test]
    async fn test_get_group_projects() {
        init();

        let config = load_config().unwrap();

        let gitlabjob = GitlabJOB::new(config);

        let response = gitlabjob.get_group_projs();

        response
            .await
            .iter()
            .for_each(|proj| debug!("Got project: {}", proj));
    }

    #[tokio::test]
    async fn test_get_prj_jobs() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let response = api.get_proj_jobs(config.project_id.unwrap(), JobScope::Canceled);

        response
            .await
            .iter()
            .for_each(|job| debug!("Got job: {}", job));
    }

    #[tokio::test]
    async fn test_get_job_info() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let response = api.get_proj_jobs(config.project_id.unwrap(), JobScope::Canceled);

        let first_jobid = response.await[0];

        let jobinfo = api.get_jobinfo(first_jobid.to_owned());


    }
}
