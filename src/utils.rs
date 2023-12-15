use std::collections::{HashMap, HashSet};

use gitlabapi::{prelude::JobInfo, PipelineID, ProjectID};
use mailsender::prelude::{MailSender, SmtpTransport};

use crate::Config;
use log::error;

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
) -> Vec<(ProjectID, Vec<PipelineID>)> {
    jobs.iter()
        .map(|(proj, jobs)| {
            (*proj, {
                let mut temp = jobs
                    .iter()
                    .map(|job| PipelineID(job.pipeline_id.unwrap()))
                    .collect::<Vec<PipelineID>>();
                temp.sort();
                temp.into_iter().skip(1).collect()
            })
        })
        .collect()
}
