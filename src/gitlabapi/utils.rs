use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use log::{debug, error, warn};

use crate::gitlabapi::GitlabJOB;

pub enum HttpMethod {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
}

pub trait ApiUtils {
    fn api_builder(&self) -> reqwest::ClientBuilder;
    fn gen_url(&self, path: &str) -> reqwest::Url;
    fn api_get(&self, url: &String) -> reqwest::RequestBuilder;
    fn api_methods(&self, url: &String, method: HttpMethod) -> reqwest::RequestBuilder;
    fn parse_json(text: String) -> Option<Value>;
}

impl ApiUtils for GitlabJOB {
    
    fn api_builder(&self) -> reqwest::ClientBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            "PRIVATE-TOKEN",
            HeaderValue::from_str(self.config.private_token.as_ref().unwrap()).unwrap(),
        );
        headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());

        reqwest::ClientBuilder::new().default_headers(headers)
    }

    fn gen_url(&self, path: &str) -> reqwest::Url {
        let new_uri = self.config.base_url.clone().unwrap() + path;

        match reqwest::Url::parse(&new_uri) {
            Ok(url) => url,
            Err(error) => {
                error!("Error while parsing url: {}", new_uri);
                panic!("Error while parsing url \"{}\": {}", new_uri, error)
            }
        }
    }

    fn api_get(&self, url: &String) -> reqwest::RequestBuilder {
        let uri = self.gen_url(url);

        debug!("Building request for {uri}");

        match self.api_builder().build() {
            Ok(getter) => getter.get(uri),
            Err(err) => {
                panic!("Coudn't construct the api caller: {}", err);
            }
        }
    }

    fn parse_json(text: String) -> Option<Value> {
        if let Ok(parsed_json) = serde_json::from_str::<Value>(&text) {
            Some(parsed_json)
        } else {
            error!("Error while parsing to json from: \n{}", text);
            None
        }
    }

    fn api_methods(&self, url: &String, method: HttpMethod) -> reqwest::RequestBuilder {
        match method {
            HttpMethod::Options => todo!(),
            HttpMethod::Get => todo!(),
            HttpMethod::Post => todo!(),
            HttpMethod::Put => todo!(),
            HttpMethod::Delete => todo!(),
            HttpMethod::Head => todo!(),
            HttpMethod::Trace => todo!(),
            HttpMethod::Connect => todo!(),
            HttpMethod::Patch => todo!(),
        }

    }
}