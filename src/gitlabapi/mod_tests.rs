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
    #[ignore = "specific group"]
    async fn test_get_specifc_group_projects() {
        init();

        let mut config = load_config().unwrap();
        config.group_id = Some(86);

        let gitlabjob = GitlabJOB::new(config);

        let response = gitlabjob.get_group_projs();

        let vec = response.await;

        vec.iter().for_each(|proj| debug!("Got project: {}", proj));

        debug!("Got {} projects", &vec.len());
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

        debug!("Job informations: {:?}", jobinfo);
    }

    #[tokio::test]
    #[ignore = "specific job"]
    async fn test_get_specif_job() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let specify_project = 513_u64;
        let specify_job = 20597_u64;

        let jobinfo = api.get_jobinfo(specify_project, specify_job).await;

        debug!("{:?}", jobinfo);
    }

    #[tokio::test]
    #[ignore = "specific pipeline"]
    async fn test_get_pipe_vars() {
        init();

        let config = load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let specify_project = 513_u64;
        let specify_pipeline = 15253_u64;

        let pipe_vars = api.get_pipe_vars(specify_project, specify_pipeline).await;

        debug!("HashMap from pipeline variables: {:?}", pipe_vars);
    }
}
