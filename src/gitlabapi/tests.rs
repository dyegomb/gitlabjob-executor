#[cfg(test)]
mod test_http {

    use std::io::Write;
    use crate::gitlabapi::prelude::*;

    fn init() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[tokio::test]
    async fn test_api_get() {
        init();
        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config);

        let response = api
            .api_get("/api/v4/projects")
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

        let config = Config::load_config().unwrap();

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

        let mut config = Config::load_config().unwrap();
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

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let response = api.get_proj_jobs(config.project_id.unwrap(), JobScope::Canceled);

        response
            .await
            .iter()
            .for_each(|job| debug!("Got: {:?}", job));
    }

    #[tokio::test]
    async fn test_get_job_info() {
        init();

        let config = Config::load_config().unwrap();

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

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let specify_project = 518_u64;
        let specify_job = 20548_u64;

        let jobinfo = api.get_jobinfo(specify_project, specify_job).await;

        debug!("Got JobInfo: {:?}", jobinfo);
    }

    #[tokio::test]
    async fn test_get_all_jobs() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config);

        let output = api.get_all_jobs(JobScope::Canceled).await;


        output.iter()
            .for_each(|job| {
                debug!("Got Job: {:?}", job)
            });
        debug!("Total jobs: {:?}", output.len());
    }

    #[tokio::test]
    #[ignore = "It only creates a manual job"]
    async fn create_job() {
        init();

        use std::env;

        let token_trigger = env::var("TOKEN_TRIGGER").unwrap_or("123456".to_owned());

        let mut form = HashMap::new();
        form.insert("token", token_trigger.as_str());
        form.insert("ref", "master");
        form.insert("variables[trigger_email]", "test@test.org");
        form.insert("variables[source_id]", "123");
        form.insert("variables[ref_source]", "master");
        form.insert("variables[PROD_TAG]", "PROD-0.0.1");

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config);

        let output = api.post_json("api/v4/projects/306/trigger/pipeline".to_owned(), form).await;

        debug!("Response: {:?}", output);
    }
    #[tokio::test]
    #[ignore = "It'll cancel a specific job"]
    async fn test_cancel_job() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let specify_project = 306_u64;
        let specify_job = 20753_u64;

        let jobinfo = api.get_jobinfo(specify_project, specify_job).await;

        if let Err(resp) = api.cancel_job(jobinfo.unwrap()).await {
            panic!("Error {}", resp)
        }

        debug!("Job canceled: {}", specify_job);

    }

    #[tokio::test]
    #[ignore = "It'll run a specific job"]
    async fn test_play_job() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let specify_project = 306_u64;
        let specify_job = 20752_u64;

        let jobinfo = api.get_jobinfo(specify_project, specify_job).await;
        debug!("To run job: {:?}", jobinfo);

        if let Err(resp) = api.play_job(jobinfo.unwrap()).await {
            panic!("Error {}", resp)
        }

        debug!("Job played: {}", specify_job);

    }

    #[tokio::test]
    #[ignore = "Specif job"]
    async fn test_new_job_status() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config.clone());

        let specify_project = 306_u64;
        let specify_job = 20752_u64;

        let jobinfo = api.get_jobinfo(specify_project, specify_job).await;

        debug!("Job last status: {}", api.get_new_job_status(jobinfo.unwrap()).await.unwrap());

    }
    #[tokio::test]
    async fn test_sort_jobs() {
        init();

        let config = Config::load_config().unwrap();

        let api = GitlabJOB::new(config);

        let output = api.get_all_jobs(JobScope::Canceled).await;

        let sorted: Vec<(u64, u64)> = output.iter().map(|job| (job.proj_id.unwrap(), job.id.unwrap())).collect();

        debug!("Sorted: {:?}", sorted);

    }
}
