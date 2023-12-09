use crate::prelude::*;

impl GitlabJOB {
    pub fn new(config: &Config) -> Self {
        GitlabJOB {
            config: config.clone(),
        }
    }
}