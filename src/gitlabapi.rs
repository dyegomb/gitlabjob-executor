//https://docs.gitlab.com/ee/api/rest/index.html
use crate::load_config::Config;
use log::{debug, error, info, warn};
use reqwest::header::{HeaderMap, HeaderValue};

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

    pub async fn get_grp_jobs(&self, scope: JobScope) -> Vec<String> {
        let uri = format!(
            "/api/v4/projects/{}/jobs?pagination=keyset&per_page=100&order_by=id&sort=asc&scope={}",
            self.config.group_id.unwrap(),
            scope.to_str()
        );
        let resp = self.api_get(&uri);

        // use serde_json
        // https://betterprogramming.pub/how-to-work-with-json-in-rust-35ddc964009e?gi=734881859642

        unimplemented!()
    }
}

#[cfg(test)]
mod test_http {
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

        let response = api.api_get(&"api/v4/projects".to_string())
                .send()
                .await
                .unwrap()
                .text()
                .await;

        debug!("{}", response.unwrap());
    }

    #[tokio::test]
    async fn test_get_group_projects() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(&config);

        let response = api.get_grp_jobs(JobScope::Success);

        // debug!("Response: {}", response);


    }
}
