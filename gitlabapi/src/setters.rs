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
pub trait JobActions<'a> {
    async fn cancel_job(&self, job: &'a JobInfo) -> Result<&'a JobInfo, String>;
    async fn play_job(&self, job: &'a JobInfo) -> Result<&'a JobInfo, String>;
}

#[async_trait]
impl<'a> JobActions<'a> for GitlabJOB {
    async fn cancel_job(&self, job: &'a JobInfo) -> Result<&'a JobInfo, String> {
        let url = format!(
            "api/v4/projects/{}/jobs/{}/cancel",
            job.proj_id.unwrap(),
            job.id.unwrap()
        );

        match self.post_json(url, Value::String("".to_owned())).await {
            Ok(_) => Ok(job),
            Err(e) => Err(format!("Error to cancel job {job}: {e}")),
        }
    }

    async fn play_job(&self, job: &'a JobInfo) -> Result<&'a JobInfo, String> {
        let url = format!(
            "api/v4/projects/{}/jobs/{}/play",
            job.proj_id.unwrap(),
            job.id.unwrap()
        );

        match self.post_json(url, Value::String("".to_owned())).await {
            Ok(_) => Ok(job),
            Err(e) => Err(format!("Error to play job {job}: {e}")),
        }
    }
}
