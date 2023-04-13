mod jobinfo;
mod mod_tests;
mod utils;
mod gitlabjob;

mod prelude {
    pub use serde_json::Value;
    pub use reqwest::header::{HeaderMap, HeaderValue};
    pub use log::{debug, error, warn};

    pub use crate::gitlabapi::gitlabjob::GitlabJOB;
    pub use crate::gitlabapi::utils::{ApiUtils, HttpMethod};
    pub use crate::gitlabapi::jobinfo::{JobInfo, JobScope};
    pub use crate::load_config::load_config;
}