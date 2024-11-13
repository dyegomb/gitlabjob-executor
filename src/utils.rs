use std::collections::{BinaryHeap, HashMap, HashSet};

use gitlabapi::prelude::*;
use mailsender::prelude::*;

use crate::MailReason;
use crate::SmtpConfig;
use log::{error, warn};

/// Build the mail relay
pub async fn mailrelay_buid(smtp_config: SmtpConfig) -> Option<SmtpTransport> {
    match smtp_config.is_valid() {
        true => match MailSender::try_new(smtp_config.to_owned()).await {
            Ok(mailer) => mailer.relay,
            Err(error) => {
                error!("{}", error);
                return None;
            }
        },
        false => None,
    };
    None
}

/// Build mail message facilitator
pub fn mail_message(job: &JobInfo, reason: MailReason, builder: &SmtpConfig) -> Message {
    let subject = match reason {
        MailReason::Duplicated => {
            format!("Job {} canceled due to duplicated pipeline", job)
        }
        MailReason::InvalidTag => format!("Job {} canceled due to invalid git tag", job),
        MailReason::ErrorToCancel => format!("Error trying to cancel job {}", job),
        MailReason::ErrorToPlay => format!("Error to start job {}", job),
        MailReason::MaxWaitElapsed => format!("Max wait time elapsed for job {}", job),
        MailReason::Status(status) => format!("Status of job {}: {}", job, status),
    };

    let to = &job.user_mail;

    builder.body_builder(subject, job.to_html(), to)
}

/// Reorder got jobs by Project id and Pipeline id skipping the first pipeline
pub fn pipelines_tocancel(
    jobs: &HashMap<ProjectID, HashSet<JobInfo>>,
) -> HashMap<ProjectID, Vec<PipelineID>> {
    let mut pipelines_tocancel = HashMap::new();
    jobs.iter()
        .map(|(proj, jobs)| {
            (*proj, {
                let mut temp = BinaryHeap::from(
                    jobs.iter()
                        .map(|job| PipelineID(job.pipeline_id.unwrap()))
                        .collect::<Vec<PipelineID>>(),
                );
                let higher = temp.peek().cloned();
                if let Some(higher) = higher {
                    temp.drain()
                        .filter(|a| a != &higher)
                        .collect::<Vec<PipelineID>>()
                } else {
                    Vec::with_capacity(0)
                }
            })
        })
        .for_each(|(key, vec)| {
            pipelines_tocancel.entry(key).or_insert(vec);
        });

    pipelines_tocancel
}

/// Check if the job must be canceled or played
pub async fn validate_jobs<'a>(
    api: &GitlabJOB,
    proj_jobs: &'a HashMap<ProjectID, HashSet<JobInfo>>,
) -> HashMap<&'a JobInfo, (bool, Option<MailReason>)> {
    let pipes_tocancel = pipelines_tocancel(proj_jobs);
    let mut checked_jobs = HashMap::new();

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
                checked_jobs.insert(job, (false, Some(MailReason::Duplicated)));
                continue;
            }
            match (job.source_id, &job.git_tag) {
                (None, None) => {
                    checked_jobs.insert(job, (true, None));
                }
                (None, Some(tag)) => {
                    let proj_tags = api.get_tags(*proj).await;
                    if proj_tags.contains(tag) {
                        checked_jobs.insert(job, (true, None));
                    } else {
                        checked_jobs.insert(job, (false, Some(MailReason::InvalidTag)));
                        warn!("The job {} will be cancelled due to invalid tag.", job);
                    }
                }
                (Some(source_proj), Some(tag)) => {
                    let proj_tags = api.get_tags(ProjectID(source_proj)).await;
                    if proj_tags.contains(tag) {
                        checked_jobs.insert(job, (true, None));
                    } else {
                        checked_jobs.insert(job, (false, Some(MailReason::InvalidTag)));
                        warn!("The job {} will be cancelled due to invalid tag.", job);
                    }
                }
                (Some(_), None) => {
                    checked_jobs.insert(job, (true, None));
                }
            }
        }
    }

    checked_jobs
}
