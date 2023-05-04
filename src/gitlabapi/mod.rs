mod gitlabjob;
mod jobinfo;
mod tests;
mod utils;

/// Specify how many concurrent tasks
pub const STREAM_BUFF_SIZE: usize = 10;

pub mod prelude {
    pub use super::STREAM_BUFF_SIZE;
    pub use crate::configloader::Config;
    pub use crate::gitlabapi::gitlabjob::GitlabJOB;
    pub use crate::gitlabapi::jobinfo::{JobInfo, JobScope};
    pub use crate::gitlabapi::utils::{ApiUtils, HttpMethod};
    pub use futures::join;
    pub use futures::stream::{self, StreamExt};
    pub use log::{debug, error, info, warn};
    pub use reqwest::header::{HeaderMap, HeaderValue};
    pub use serde_json::Value;
    pub use std::collections::{HashMap, HashSet};
}
