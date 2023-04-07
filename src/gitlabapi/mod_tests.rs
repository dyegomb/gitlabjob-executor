#[cfg(test)]
mod test_http {
    use std::io::Write;

    use crate::gitlabapi::*;
    use crate::load_config;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[tokio::test]
    async fn test_api_get() {
        init();
        let config = load_config().unwrap();

        let api = GitlabJOB::new(config);

        let response = api
            .api_get(&"/api/v4/projects".to_string())
            .send()
            .await
            .unwrap()
            .text()
            .await;

        let parsed_json: Value = serde_json::from_str(response.as_deref().unwrap()).unwrap();

        parsed_json.as_array().iter().for_each(|projects| {
            projects.iter().for_each(|proj| {
                debug!("Project ID: {}", proj["id"]);
                debug!("Project links: {}", proj["_links"]);
            });
        });

        let _ = std::fs::File::create("/tmp/test.json")
            .unwrap()
            .write(response.unwrap().as_bytes());
    }

    #[tokio::test]
    async fn test_get_group_projects() {
        init();

        let config = load_config().unwrap();

        let gitlabjob = GitlabJOB::new(config);

        let response = gitlabjob.get_group_projs();

        response
            .await
            .iter()
            .for_each(|proj| debug!("Got project: {}", proj));
    }

    #[tokio::test]
    async fn test_get_prj_jobs() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let response = api.get_proj_jobs(config.project_id.unwrap(), JobScope::Canceled);

        response
            .await
            .iter()
            .for_each(|job| debug!("Got: {:?}", job));
    }

    #[tokio::test]
    async fn test_get_proj_info() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let response = api.get_proj_info(config.project_id.unwrap()).await;

        debug!("Project infos: {:?}", response);
    }

    #[tokio::test]
    async fn test_get_job_info() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let response = api
            .get_proj_jobs(config.project_id.unwrap(), JobScope::Canceled)
            .await;

        let job_test = response.iter().next().unwrap();

        let jobinfo = api.get_jobinfo(job_test.0.to_owned(), job_test.1[0]).await;

        debug!("{:?}", jobinfo);
    }

    #[tokio::test]
    async fn test_get_specif_job() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let jobinfo = api.get_jobinfo(513_u64, 20597_u64).await;

        debug!("{:?}", jobinfo);
    }

    #[tokio::test]
    async fn test_get_pipe_vars() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let pipe_vars = api.get_pipe_vars(513_u64, 15253_u64).await;

        debug!("HashMap from pipeline variables: {:?}", pipe_vars);
    }
}
