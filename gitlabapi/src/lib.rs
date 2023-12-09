mod getters;

pub use configloader::Config;

/// Specify how many concurrent tasks
pub const STREAM_BUFF_SIZE: usize = 15;

mod prelude {
    pub use super::GitlabJOB;
    pub use super::Config;
}

/// API caller configured from `Config` module.
pub struct GitlabJOB {
    pub config: Config,
}

