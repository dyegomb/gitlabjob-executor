mod getters;
mod utils;

pub use configloader::Config;

/// Specify how many concurrent tasks
pub const STREAM_BUFF_SIZE: usize = 15;

mod prelude {
    pub use super::GitlabJOB;
    pub use super::Config;
    pub use log::{debug, error, warn};
    pub use serde_json::Value;
}

/// API caller configured from `Config` module.
pub struct GitlabJOB {
    pub config: Config,
}

