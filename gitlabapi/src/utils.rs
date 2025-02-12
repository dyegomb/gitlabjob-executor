use reqwest::header::{HeaderMap, HeaderValue};

use crate::prelude::*;

#[non_exhaustive]
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

impl GitlabJOB {
    pub fn api_caller(&self, url: &str, method: HttpMethod) -> reqwest::RequestBuilder {
        let uri = self.gen_url(url);

        match self.api_builder().build() {
            Ok(http_client) => match method {
                HttpMethod::Get => http_client.request(reqwest::Method::GET, uri),
                HttpMethod::Post => http_client.request(reqwest::Method::POST, uri),
                _ => http_client.request(reqwest::Method::GET, uri),
            },

            Err(error) => {
                error!("Couldn't construct the api caller: {}", error);
                std::process::exit(11)
            }
        }
    }

    pub fn api_get(&self, url: &str) -> reqwest::RequestBuilder {
        self.api_caller(url, HttpMethod::Get)
    }

    pub fn gen_url(&self, path: &str) -> reqwest::Url {
        let base = self.config.base_url.clone().unwrap();

        let new_uri = if path.starts_with('/') || base.ends_with('/') {
            base + path
        } else {
            base + "/" + path
        };

        match reqwest::Url::parse(&new_uri) {
            Ok(url) => url,
            Err(error) => {
                error!("Error while parsing url \"{}\": {}", new_uri, error);
                std::process::exit(12)
            }
        }
    }

    pub fn api_builder(&self) -> reqwest::ClientBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            "PRIVATE-TOKEN",
            HeaderValue::from_str(self.config.private_token.as_ref().unwrap()).unwrap(),
        );
        headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());
        headers.insert(
            "Content-type",
            HeaderValue::from_str("application/json; charset=utf-8").unwrap(),
        );

        reqwest::ClientBuilder::new().default_headers(headers)
    }

    pub fn parse_json(text: String) -> Result<Value, String> {
        if let Ok(parsed_json) = serde_json::from_str::<Value>(&text) {
            Ok(parsed_json)
        } else {
            Err(format!("Error while parsing to json from: \n{}", text))
        }
    }

    pub fn api_post(&self, url: &str, json: Value) -> reqwest::RequestBuilder {
        // let post_json = serde_json::json!(form);

        debug!("Post JSON: {}", json);

        self.api_caller(url, HttpMethod::Post).json(&json)
    }
}
