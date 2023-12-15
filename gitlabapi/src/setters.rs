use async_trait::async_trait;

use crate::prelude::*;

impl GitlabJOB {
    /// Post facilitator with serde_json::Value as post body
    async fn post_json(&self, url: String, json: Value) -> Result<Value, String> {
        let resp = self.api_post(url.as_str(), json);

        match resp.send().await {
            Err(e) => Err(format!("Error while posting to {url}: {e}")),
            Ok(response) => {
                debug!("HTTP Response Headers: {:?}", response.headers());
                debug!("HTTP Response Status: {:?}", response.status());
                debug!("HTTP Response Url: {:?}", response.url());
                match response.text().await {
                    Err(e) => Err(e.to_string()),
                    Ok(text) => Self::parse_json(text),
                }
            }
        }
    }
}

#[async_trait]
trait JobActions {
    async fn cancel_job(&self, job: &JobInfo) -> Result<(), String>;
    async fn play_job(&self, job: &JobInfo) -> Result<(), String>;
}

#[async_trait]
impl JobActions for GitlabJOB {
    async fn cancel_job(&self, job: &JobInfo) -> Result<(), String> {
        todo!()
    }

    async fn play_job(&self, job: &JobInfo) -> Result<(), String> {
        let url = format!(
            "api/v4/projects/{}/jobs/{}/play",
            job.proj_id.unwrap(),
            job.id.unwrap()
        );

        let resp = self.post_json(url, Value::String("".to_owned()));

        if let Ok(response) = resp.await {
            debug!("Job played: {:?}", response);
            return Ok(());
        }

        Err(format!("Couldn't play job {:?}", job))
    }
}
