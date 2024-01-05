// mod crate::getters_traits;

use std::collections::{HashMap, HashSet};

use crate::prelude::*;
// use crate::getters_traits::*;

impl GitlabJOB {
    /// Get a tuple from an option with serde_json::Value and number of pages as u64
    pub async fn get_json(&self, url: &String) -> Result<(Value, u64), String> {
        let resp = self.api_get(url);
        debug!("Getting json from: {url}");

        match resp.send().await {
            Err(e) => Err(format!("Error while getting {url}: {}", e)),
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
                    Err(e) => Err(e.to_string()),
                }
            }
        }
    }

    /// Recover trigger variables from a Gitlab pipeline.
    pub async fn get_pipe_vars(&self, projid: u64, pipelineid: u64) -> HashMap<String, String> {
        let uri = format!("/api/v4/projects/{projid}/pipelines/{pipelineid}/variables");

        let mut hashmap_out: HashMap<String, String> = HashMap::new();

        let mut new_uri;
        let mut current_page = 1;

        loop {
            new_uri = format!("{}?per_page=100&page={}", &uri, current_page);
            let num_pages;

            if let Ok((vars_obj, pages)) = self.get_json(&new_uri).await {
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

    /// Get projects ids from a Gitlab group
    pub async fn get_projs(&self, groupid: GroupID) -> HashSet<u64> {
        // if self.config.group_id.is_none() {
        //     return vec![];
        // };

        let base_uri = format!(
            "/api/v4/groups/{}/projects?pagination=keyset&simple=true&per_page=100&order_by=id&sort=asc",
            // self.config.group_id.unwrap()
            groupid.0
        );

        let mut vec_projs = HashSet::new();

        let mut current_page = 1;

        loop {
            let new_uri = format!("{}&page={}", &base_uri, current_page);
            let num_pages;

            if let Ok((json, total_pages)) = self.get_json(&new_uri).await {
                num_pages = total_pages;
                if let Some(vec_json) = json.as_array() {
                    vec_json.iter().for_each(|proj| {
                        if let Some(val) = proj["id"].as_u64() {
                            vec_projs.insert(val);
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

    /// Inspect a project for its git tags.
    pub async fn get_tags(&self, id: ProjectID) -> Vec<String> {
        let url = format!("api/v4/projects/{}/repository/tags?order_by=updated", id.0);

        let mut got_tags = vec![];

        if let Ok((resp, _)) = self.get_json(&url).await {
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

    /// Get current status of a job
    pub async fn get_status(&self, job: &JobInfo) -> JobScope {
        let uri = format!(
            "/api/v4/projects/{}/jobs/{}",
            job.proj_id.unwrap(),
            job.id.unwrap()
        );

        match self.get_json(&uri).await {
            Ok((resp, _)) => {
                if let Some(json) = resp.get("status") {
                    json.to_string().trim().to_owned().into()
                } else {
                    JobScope::Invalid
                }
            }
            Err(_) => JobScope::Invalid,
        }
    }
}
