mod jobinfo;
mod tests;
mod utils;
mod gitlabjob;

pub mod prelude {
    pub use serde_json::Value;
    pub use reqwest::header::{HeaderMap, HeaderValue};
    pub use log::{debug, error, warn};
    pub use std::collections::{HashMap, HashSet};
    pub use futures::join;
    pub use futures::stream::{ self, StreamExt};
    pub use crate::gitlabapi::gitlabjob::GitlabJOB;
    pub use crate::gitlabapi::utils::{ApiUtils, HttpMethod};
    pub use crate::gitlabapi::jobinfo::{JobInfo, JobScope};
    pub use crate::configloader::Config;
}

