//https://docs.gitlab.com/ee/api/rest/index.html
use crate::load_config::Config;
use log::{debug, info, warn, error};
use reqwest::header::{HeaderValue, HeaderMap};

// extern crate reqwest;


pub enum JobScope {
    Created, Pending, Running, Failed, 
    Success, Canceled, Skipped, WaitingForResource, 
    Manual,
}

fn api_builder(config: &Config) -> reqwest::ClientBuilder {

    let mut headers = HeaderMap::new();
    headers.insert("PRIVATE-TOKEN", HeaderValue::from_str(config.private_token.as_ref().unwrap()).unwrap());
    headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());

    reqwest::ClientBuilder::new()
        .default_headers(headers)

}

fn api_get(config: &Config, url: &'static str) -> reqwest::RequestBuilder {

    let uri = config.base_url.as_ref().unwrap().clone() + url;

    match api_builder(config).build() {
        Ok(getter) => getter.get(uri),
        Err(err) => {
            panic!("Coudn't construct the api getter. {}", err);
        },
    }
}

pub fn get_grp_jobs(config: &Config, scope:JobScope) -> Vec<String> {
    let uri = format!("/api/v4/projects/{}/jobs?pagination=keyset&per_page=100&order_by=id&sort=asc", config.group_id.unwrap());

    unimplemented!()
}


#[cfg(test)]
mod test_http {
    use crate::load_config;

    use super::*;
    use futures::executor::block_on;

    fn init() {
        let _ = env_logger::builder()
            // Include all events in tests
            .filter_level(log::LevelFilter::max())
            // Ensure events are captured by `cargo test`
            .is_test(true)
            // Ignore errors initializing the logger if tests race to configure it
            .try_init();
    }

    #[test]
    fn test_api_get() {
        init();
        let config = load_config().unwrap();

        let req = api_get(&config, "api/v4/projects").send();

        let resp = block_on(req).unwrap();
        let text = block_on(resp.text());
        println!("{}", text.unwrap());
        // println!("{:?}", req);
        // futures::executor::block_on(req);
        // match req() {
        //     resp => println!("{:?}", block_on(resp)),
        //     _ => (),
        // }


    }

}