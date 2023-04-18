use crate::gitlabapi::prelude::*;

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
    fn api_caller(&self, url: &str, method: HttpMethod) -> reqwest::RequestBuilder;
    fn api_get(&self, url: &str) -> reqwest::RequestBuilder;
    fn api_post(&self, url: &str, form: HashMap<&str, &str>) -> reqwest::RequestBuilder;

    fn parse_json(text: String) -> Option<Value> {
        if let Ok(parsed_json) = serde_json::from_str::<Value>(&text) {
            Some(parsed_json)
        } else {
            error!("Error while parsing to json from: \n{}", text);
            None
        }
    }
}

impl ApiUtils for GitlabJOB {
    fn api_builder(&self) -> reqwest::ClientBuilder {
        let mut headers = HeaderMap::new();
        headers.insert(
            "PRIVATE-TOKEN",
            HeaderValue::from_str(self.config.private_token.as_ref().unwrap()).unwrap(),
        );
        headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());
        headers.insert("Content-type", HeaderValue::from_str("application/json; charset=utf-8").unwrap());

        reqwest::ClientBuilder::new().default_headers(headers)
    }

    fn gen_url(&self, path: &str) -> reqwest::Url {
        let base = self.config.base_url.clone().unwrap();

        let new_uri = if path.starts_with('/') || base.ends_with('/') {
            base + path
        } else {
            base + "/" + path
        };

        match reqwest::Url::parse(&new_uri) {
            Ok(url) => url,
            Err(error) => {
                error!("Error while parsing url: {}", new_uri);
                panic!("Error while parsing url \"{}\": {}", new_uri, error)
            }
        }
    }

    fn api_caller(&self, url: &str, method: HttpMethod) -> reqwest::RequestBuilder {
        let uri = self.gen_url(url);

        match self.api_builder().build() {
            Ok(http_client) => {
                debug!("API CALLER: {:?}", http_client);
                match method {
                    HttpMethod::Options => http_client.request(reqwest::Method::OPTIONS, uri),
                    HttpMethod::Get => http_client.request(reqwest::Method::GET, uri),
                    HttpMethod::Post => http_client.request(reqwest::Method::POST, uri),
                    HttpMethod::Put => http_client.request(reqwest::Method::PUT, uri),
                    HttpMethod::Delete => http_client.request(reqwest::Method::DELETE, uri),
                    HttpMethod::Head => http_client.request(reqwest::Method::HEAD, uri),
                    HttpMethod::Trace => http_client.request(reqwest::Method::TRACE, uri),
                    HttpMethod::Connect => http_client.request(reqwest::Method::CONNECT, uri),
                    HttpMethod::Patch => http_client.request(reqwest::Method::PATCH, uri),
                }
            }
            Err(error) => {
                panic!("Couldn't construct the api caller: {}", error)
            }
        }
    }

    fn api_get(&self, url: &str) -> reqwest::RequestBuilder {
        self.api_caller(url, HttpMethod::Get)
    }

    fn api_post(&self, url: &str, form: HashMap<&str, &str>) -> reqwest::RequestBuilder {

        let post_body = serde_json::to_vec(&form).unwrap_or(vec![]);

        self.api_caller(url, HttpMethod::Post)
            .body(post_body)
    }
}
