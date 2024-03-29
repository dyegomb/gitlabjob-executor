use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use futures::join;
use futures::stream::{self, StreamExt};

use crate::prelude::*;

#[async_trait]
pub trait Getjobs<T, R> {
    type R;
    async fn get_jobs(&self, id: T, scope: JobScope) -> Self::R;
}

#[async_trait]
impl Getjobs<ProjectID, HashMap<ProjectID, HashSet<JobInfo>>> for GitlabJOB {
    type R = HashMap<ProjectID, HashSet<JobInfo>>;
    async fn get_jobs(&self, id: ProjectID, scope: JobScope) -> Self::R {
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
            if let Ok((json, pages)) = parse_json {
                num_pages = pages;
                match json.as_array() {
                    Some(vec_json) => {
                        vec_json.iter().for_each(|proj| {
                            if let Some(val) = proj["id"].as_u64() {
                                map_jobs.push(val)
                            } else {
                                warn!("Unable to get jobs for project {}", id.0);
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

        let projid_jobid_tuple: Vec<(ProjectID, JobID)> =
            map_jobs.iter().map(|a| (id, JobID(*a))).collect();

        let mut stream_jobs = stream::iter(projid_jobid_tuple)
            .map(|(projid, jobid)| async move { (projid, self.get_info((projid, jobid)).await) })
            .buffer_unordered(STREAM_BUFF_SIZE)
            .fuse();

        let mut proj_jobs: HashMap<ProjectID, HashSet<JobInfo>> = HashMap::new();
        while let Some((projid, jobinfo)) = stream_jobs.next().await {
            if let Ok(jobinfo) = jobinfo {
                proj_jobs
                    .entry(projid)
                    .and_modify(|jobs| {
                        jobs.insert(jobinfo.clone());
                    })
                    .or_insert(HashSet::from([jobinfo]));
            }
        }

        proj_jobs
    }
}

/// Scans scoped jobs orderning by project ids.
#[async_trait]
impl Getjobs<GroupID, HashMap<ProjectID, HashSet<JobInfo>>> for GitlabJOB {
    type R = HashMap<ProjectID, HashSet<JobInfo>>;

    async fn get_jobs(&self, id: GroupID, scope: JobScope) -> Self::R {
        let projects = self.get_projs(id).await;

        let stream_projects = stream::iter(projects)
            .map(|proj| async move { self.get_jobs(ProjectID(proj), scope).await })
            .buffer_unordered(STREAM_BUFF_SIZE)
            .fuse();
        tokio::pin!(stream_projects);

        let mut proj_jobs = HashMap::new();

        while let Some(hashmap) = stream_projects.next().await {
            proj_jobs.extend(hashmap)
        }

        proj_jobs
    }
}

#[async_trait]
pub trait GetInfo<T, R> {
    type R;
    async fn get_info(&self, id: T) -> Self::R;
}

/// Get some informations from a project. *namely the project "name"*.
#[async_trait]
impl GetInfo<ProjectID, HashMap<String, String>> for GitlabJOB {
    type R = HashMap<String, String>;

    async fn get_info(&self, id: ProjectID) -> Self::R {
        let uri = format!("/api/v4/projects/{}", id.0);

        let parse_json = self.get_json(&uri).await;
        let mut hash_map: HashMap<String, String> = HashMap::new();
        if let Ok((json, _)) = parse_json {
            if let Some(name) = json["name"].as_str() {
                hash_map.insert("name".to_owned(), name.to_owned());
            }
        }
        hash_map
    }
}

/// The actual getter of job's informations, returning as `Jobinfo` struct.
#[async_trait]
impl GetInfo<(ProjectID, JobID), Result<JobInfo, String>> for GitlabJOB {
    type R = Result<JobInfo, String>;

    async fn get_info(&self, id: (ProjectID, JobID)) -> Self::R {
        let projid = id.0;
        let jobid = id.1;

        let uri = format!("/api/v4/projects/{}/jobs/{}", projid.0, jobid.0);

        let (parse_json, project_infos) = join!(self.get_json(&uri), self.get_info(projid));

        let mut jobinfo = JobInfo {
            id: Some(jobid.0),
            proj_id: Some(projid.0),
            ..Default::default()
        };

        if let Ok((json, _)) = parse_json {
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
                    variables = self.get_pipe_vars(projid.0, pipe_id).await;
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

            return Ok(jobinfo);
        };

        Err("Failed to gather job infos".to_string())
    }
}
