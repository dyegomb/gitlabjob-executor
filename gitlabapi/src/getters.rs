use crate::prelude::*;

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

}
