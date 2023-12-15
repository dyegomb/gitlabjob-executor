#[cfg(test)]
mod integration_tests {
    use crate::*;
    // use std::io::Write;
    use log::debug;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[tokio::test]
    async fn test_pipelines_to_cancel() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(&config);
        let proj = ProjectID(config.project_id.unwrap());

        let response = api.get_jobs(proj, JobScope::Canceled).await;

        response.iter().for_each(|(project, jobs)| {
            debug!("Project {} has {} pipelines.", project.0, jobs.len())
        });

        debug!("Got {} projects", response.values().len());
        let to_cancel = utils::pipelines_tocancel(&response);
        to_cancel.iter().for_each(|(proj, jobs)| {
            debug!(
                "For project {}, {} pipelines will be canceled",
                proj.0,
                jobs.len()
            )
        });
        debug!("{:?}", to_cancel);
    }
}
