use std::collections::{HashMap, HashSet};

use gitlabapi::prelude::*;
use mailsender::prelude::*;

use crate::{Config, MailReason};
use log::{error, warn};

/// Build the mail relay
pub async fn mailrelay_buid(config: Config) -> Option<SmtpTransport> {
    match &config.smtp {
        Some(smtp) => match smtp.is_valid() {
            true => match MailSender::try_new(smtp.to_owned()).await {
                Ok(mailer) => mailer.relay,
                Err(error) => {
                    error!("{}", error);
                    None
                }
            },
            false => None,
        },
        None => None,
    }
}

/// Reorder got jobs by Project id and Pipeline id skipping the first pipeline
pub fn pipelines_tocancel(
    jobs: &HashMap<ProjectID, HashSet<JobInfo>>,
) -> HashMap<ProjectID, Vec<PipelineID>> {
    let mut pipelines_tocancel = HashMap::new();
    jobs.iter()
        .map(|(proj, jobs)| {
            (*proj, {
                let mut temp = jobs
                    .iter()
                    .map(|job| PipelineID(job.pipeline_id.unwrap()))
                    .collect::<Vec<PipelineID>>();
                temp.sort();
                temp.reverse();
                temp.into_iter().skip(1).collect::<Vec<PipelineID>>()
            })
        })
        .for_each(|(key, vec)| {
            pipelines_tocancel.entry(key).or_insert(vec);
        });

    pipelines_tocancel
}

pub async fn validate_jobs<'a>(
    api: &GitlabJOB,
    proj_jobs: &'a HashMap<ProjectID, HashSet<JobInfo>>,
    // pipelines_tocancel: &HashMap<&ProjectID, Vec<PipelineID>>,
    // source_tags: &Vec<String>,
) -> Vec<(bool, &'a JobInfo, Option<MailReason>)> {
    let pipes_tocancel = pipelines_tocancel(proj_jobs);
    let mut checked_jobs = vec![];

    for (proj, jobs) in proj_jobs {
        for job in jobs {
            if pipes_tocancel
                .get(proj)
                .unwrap()
                .contains(&PipelineID(job.pipeline_id.unwrap()))
            {
                warn!(
                    "The job {} will be canceled due to duplicated pipelines",
                    job
                );
                checked_jobs.push((false, job, Some(MailReason::Duplicated)));
                continue;
            }
            match (job.source_id, &job.git_tag) {
                (None, None) => checked_jobs.push((true, job, None)),
                (None, Some(tag)) => {
                    let proj_tags = api.get_tags(*proj).await;
                    if proj_tags.contains(tag) {
                        checked_jobs.push((true, job, None));
                    } else {
                        checked_jobs.push((false, job, Some(MailReason::InvalidTag)));
                        warn!("The job {} will be cancelled due to invalid tag.", job);
                    }
                }
                (Some(source_proj), Some(tag)) => {
                    let proj_tags = api.get_tags(ProjectID(source_proj)).await;
                    if proj_tags.contains(tag) {
                        checked_jobs.push((true, job, None));
                    } else {
                        checked_jobs.push((false, job, Some(MailReason::InvalidTag)));
                        warn!("The job {} will be cancelled due to invalid tag.", job);
                    }
                }
                (Some(_), None) => checked_jobs.push((true, job, None)),
            }
        }
    }

    checked_jobs
}
