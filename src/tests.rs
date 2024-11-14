#[cfg(test)]
mod integration_tests {
    use crate::*;
    // use std::io::Write;
    use log::debug;
    use std::process::exit;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[tokio::test]
    #[ignore = "pipeline cration"]
    async fn create_pipeline() {
        // TESTE_TOKENTRIG="token" cargo test --package gitlabjob --bin gitlabjob -- tests::integration_tests::create_pipeline \
        // --exact --nocapture --ignored

        // curl -X POST --fail -F "token=$totken" -F "ref=master" -F "variables[PROD_TAG]=PROD-1.1" https://gitlab.com/api/v4/projects/***PROJID***/trigger/pipeline
        use std::env;

        let token_trigger = match env::var("TESTE_TOKENTRIG") {
            Ok(value) => value,
            Err(_) => {
                error!("No token to trigger a new job");
                exit(1)
            }
        };

        init();

        let config = Config::load_config().unwrap();
        let api = GitlabJOB::new(&config);

        let url = format!(
            "api/v4/projects/{}/trigger/pipeline",
            config.project_id.unwrap_or(0)
        );

        let json_post = serde_json::json!({
                    "token": token_trigger,
                    "ref": "main",
                    // "variables": {"trigger_email":"teste@test.tst"},
                    // "variables": {"ref_source":"master"},
                    // "variables": {"source_id":config.project_id.unwrap_or(0).to_string()},
                    "variables": {"PROD_TAG": "Test-test-1.0.0",
                        "trigger_email":"teste@test.tst"},
                }
        );

        match api.post_json(url, json_post).await {
            Ok(resp) => debug!("New pipeline created:\n{:?}", resp),
            Err(error) => {
                error!("Failed to create new pipeline: {}", error);
                exit(1)
            }
        }
    }

    #[tokio::test]
    async fn test_pipelines_to_cancel() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(&config);
        let proj = ProjectID(config.project_id.unwrap());

        let response = api.get_jobs(proj, JobScope::Manual).await;

        response.iter().for_each(|(project, jobs)| {
            debug!("Project {} has {} pipelines.", project.0, jobs.len())
        });

        debug!("Got {} projects", response.values().len());
        let to_cancel = utils::pipelines_tocancel(&response);
        to_cancel.iter().for_each(|(proj, pipes)| {
            debug!(
                "For project {}, {} jobs will be canceled",
                proj.0,
                pipes.len()
            )
        });
        debug!("{:?}", to_cancel);
    }

    #[tokio::test]
    #[ignore = "send email"]
    async fn test_email() {
        init();

        let config = Config::load_config().unwrap().smtp;

        let mail_relay_handle = tokio::spawn(utils::mailrelay_build(config.clone().unwrap()));

        let test_job = JobInfo::default();

        let message = utils::mail_message(
            &test_job,
            MailReason::ErrorToPlay,
            &config.unwrap_or_default(),
        );

        let mail_relay = mail_relay_handle.await.unwrap_or_default();

        if let Some(mailer) = mail_relay {
            match mailer.send(&message) {
                Ok(resp) => debug!("{:?}", resp),
                Err(resp) => error!("{}", resp),
            };
        }
    }
}
