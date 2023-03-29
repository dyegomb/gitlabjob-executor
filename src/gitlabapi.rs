//https://docs.gitlab.com/ee/api/rest/index.html
use crate::load_config::Config;
use log::{debug, error, info, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

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

impl JobScope {
    fn to_str(&self) -> &'static str {
        match self {
            JobScope::Created => "created",
            JobScope::Pending => "pending",
            JobScope::Running => "running",
            JobScope::Failed => "failed",
            JobScope::Success => "success",
            JobScope::Canceled => "canceled",
            JobScope::Skipped => "skipped",
            JobScope::WaitingForResource => "waiting_for_resource",
            JobScope::Manual => "manual",
        }
    }
}

pub struct GitlabJOB {
    config: Config,
}

impl GitlabJOB {
    pub fn new(config: &Config) -> Self {
        GitlabJOB {
            config: config.clone(),
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

    pub fn get_group_projs(&self) -> Vec<String> {
        todo!()
    }

    pub async fn get_prj_jobs(
        &self,
        project: usize,
        scope: JobScope,
    ) -> Vec<Value> {
        let uri = format!(
            "/api/v4/projects/{}/jobs?per_page=100&order_by=id&sort=asc&scope={}",
            project.to_string(),
            scope.to_str()
        );

        let resp = self.api_get(&uri).send().await.unwrap().text().await;

        // let mut tmp_vec = vec![];
        let parse_json;
        if let Ok(got_resp) = resp {
            parse_json = serde_json::from_str::<Value>(&got_resp);
        } else {
            panic!("Error parsing json response from {}", &uri);
        };

        let mut vec_jobs: Vec<Value> = vec![];

        if let Ok(json) = parse_json {
            match json.as_array() {
                Some(vec_json) => {
                    vec_json.iter().for_each(|proj| {
                        let val = proj.clone();
                        vec_jobs.push(val);
                    });
                }
                None => {
                    debug!("No jobs found for {}", uri);
                    return vec![]
                }
            }
        };

        // debug!("First job gotten: {:?}", &vec_jobs[0]);

        // vec_jobs.iter().for_each(|v| println!("{v}"));

        vec_jobs
    }
}

#[cfg(test)]
mod test_http {
    use std::io::Write;

    // use serde::Deserializer;

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

        let api = GitlabJOB::new(&config);

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

        let api = GitlabJOB::new(&config);

        // let response = api.get_grp_jobs(JobScope::Success);

        // debug!("Response: {}", response);
    }

    #[tokio::test]
    async fn test_get_prj_jobs() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(&config);

        let response = api.get_prj_jobs(config.project_id.unwrap(), JobScope::Canceled);

        response.await
            .iter()
            .for_each(|job| {debug!("Got job: {}", job["id"])});
    }
}
