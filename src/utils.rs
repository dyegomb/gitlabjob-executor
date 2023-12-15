use std::collections::{HashMap, HashSet};

use gitlabapi::{prelude::JobInfo, PipelineID, ProjectID};
use mailsender::prelude::{MailSender, SmtpTransport};

use crate::Config;
use log::error;

/// Build the mail relay
pub async fn mailrelay_buid(config: &Config) -> Option<SmtpTransport> {
    match &config.smtp {
        Some(smtp) => match smtp.is_valid() {
            true => match MailSender::try_new(smtp.clone()).await {
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

/// Reorder got jobs by Project id and Pipeline id
pub fn pipelines_tocancel(
    jobs: &HashMap<ProjectID, HashSet<JobInfo>>,
) -> Vec<(ProjectID, Vec<PipelineID>)> {
    jobs.iter()
        .map(|(proj, jobs)| {
            (
                *proj,
                jobs.iter()
                    .map(|job| PipelineID(job.pipeline_id.unwrap()))
                    .collect::<Vec<PipelineID>>(),
            )
        })
        .collect()
}
