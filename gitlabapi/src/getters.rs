use std::collections::HashMap;
use async_trait::async_trait;

use crate::prelude::*;

impl GitlabJOB {
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

    /// Get projects ids from a Gitlab group
    pub async fn get_projs(&self, groupid: GroupID) -> Vec<u64> {
        // if self.config.group_id.is_none() {
        //     return vec![];
        // };

        let base_uri = format!(
            "/api/v4/groups/{}/projects?pagination=keyset&simple=true&per_page=100&order_by=id&sort=asc",
            // self.config.group_id.unwrap()
            groupid.0
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

}

#[async_trait]
pub trait Getjobs<T, R> {
    type R;
    async fn get_jobs(&self, id: T, scope: JobScope) -> Self::R;
}

#[async_trait]
impl Getjobs<ProjectID, Vec<u64>> for GitlabJOB{ 
    type R = Vec<u64>;
    async fn get_jobs(&self, id: ProjectID , scope: JobScope) -> Self::R {
        let uri = format!(
            "/api/v4/projects/{}/jobs?per_page=100&order_by=id&sort=asc&scope={}",
            id.0, scope
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
                                error!("Unable to get jobs for project {}", id.0);
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
}

#[async_trait]
impl Getjobs<ProjectID, HashMap<u64, Vec<JobInfo>>> for GitlabJOB {
    type R = HashMap<u64, Vec<JobInfo>>;

    async fn get_jobs(&self, id: ProjectID, scope: JobScope) -> Self::R {
        todo!()
    }

}

    // /// Scans scoped jobs orderning by project ids.
    // pub async fn get_jobs_by_project(&self, scope: JobScope) -> HashMap<u64, Vec<JobInfo>> {
    //     let projects = self.get_inner_projs().await;

    //     let stream_projects = stream::iter(&projects)
    //         .map(|proj| async move { (proj, self.get_proj_jobs(*proj, scope).await) })
    //         .buffer_unordered(STREAM_BUFF_SIZE)
    //         .fuse();
    //     tokio::pin!(stream_projects);

    //     let mut projid_jobid_tuple: Vec<(u64, u64)> = vec![];
    //     while let Some((proj, mut jobs)) = stream_projects.next().await {
    //         jobs.sort();
    //         jobs.reverse();
    //         jobs.iter().for_each(|jobid| {
    //             projid_jobid_tuple.push((*proj, *jobid));
    //         });
    //     }

    //     let mut stream_jobs = stream::iter(&projid_jobid_tuple)
    //         .map(|(projid, jobid)| async move { (projid, self.get_jobinfo(*projid, *jobid).await) })
    //         .buffer_unordered(STREAM_BUFF_SIZE)
    //         .fuse();

    //     let mut proj_jobs: HashMap<u64, Vec<JobInfo>> = HashMap::new();
    //     while let Some((projid, jobinfo)) = stream_jobs.next().await {
    //         if let Some(jobinfo) = jobinfo {
    //             proj_jobs
    //                 .entry(*projid)
    //                 .and_modify(|jobs| {
    //                     jobs.push(jobinfo.clone());
    //                 })
    //                 .or_insert(Vec::from([jobinfo]));
    //         }
    //     }

    //     proj_jobs
    // }

trait GetProjects {
    
}