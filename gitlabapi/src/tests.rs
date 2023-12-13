#[cfg(test)]
mod test_http {

    use crate::prelude::*;
    use std::io::Write;

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

        let api = GitlabJOB::new(&config);

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

        let gitlabjob = GitlabJOB::new(&config);
        let groupid = gitlabjob.config.group_id.unwrap();

        let response = gitlabjob.get_projs(GroupID(groupid)).await;

        response
            .iter()
            .for_each(|proj| debug!("Got project: {}", proj));
    }
}
