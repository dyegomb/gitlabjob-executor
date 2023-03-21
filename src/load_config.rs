use log::{debug, error};
use merge::Merge;
use serde::Deserialize;
use std::fmt::Error;

extern crate envy;
extern crate merge;
extern crate toml;

/// Possible configurations
#[derive(Deserialize, Debug, Merge, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    group_id: Option<usize>,
    project_id: Option<usize>,
    private_token: Option<String>,
    base_url: Option<String>,
    smtp_server: Option<String>,
    smtp_user: Option<String>,
    smtp_from: Option<String>,
    smtp_to: Option<String>,
    smtp_subject: Option<String>,
}

impl Config {
    fn new() -> Self {
        Config {
            group_id: None,
            project_id: None,
            private_token: None,
            base_url: None,
            smtp_server: None,
            smtp_user: None,
            smtp_from: None,
            smtp_to: None,
            smtp_subject: None,
        }
    }
}

/// Get configurations from environment or from file
pub fn load_config() -> Result<Config, Error> {
    let mut config;

    match envy::from_env::<Config>() {
        Ok(env_config) => {
            config = env_config;
        }
        Err(err) => {
            panic!("Couldn't load settings from environment: {}", err);
        }
    };

    let env_file = std::env::var("ENV_FILE").unwrap_or(".env".to_string());

    if let Ok(content) = std::fs::read_to_string(&env_file) {
        debug!("To read file {}.", &env_file);

        match toml::from_str::<Config>(&content) {
            Ok(toml_text) => {
                let config_file: Config = toml_text;
                config.merge(config_file);
            }
            Err(err) => {
                error!("Couldn't read file {}: {}", env_file, err);
            }
        };
    };

    Ok(config)
}

#[cfg(test)]
mod test_load_config {
    use super::*;

    fn env_cleaner() {
        std::env::remove_var("group_id".to_uppercase());
        std::env::remove_var("project_id".to_uppercase());
        std::env::remove_var("private_token".to_uppercase());
        std::env::remove_var("base_url".to_uppercase());
        std::env::remove_var("smtp_server".to_uppercase());
        std::env::remove_var("smtp_user".to_uppercase());
        std::env::remove_var("smtp_from".to_uppercase());
        std::env::remove_var("smtp_to".to_uppercase());
        std::env::remove_var("smtp_subject".to_uppercase());
        std::env::remove_var("group_id");
        std::env::remove_var("project_id");
        std::env::remove_var("private_token");
        std::env::remove_var("base_url");
        std::env::remove_var("smtp_server");
        std::env::remove_var("smtp_user");
        std::env::remove_var("smtp_from");
        std::env::remove_var("smtp_to");
        std::env::remove_var("smtp_subject");
    }

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
    fn test_all() {
        // running all tests in same place cause concurrency problems
        init();
        env_cleaner();

        let confs = load_config().unwrap();
        let config_new = Config::new();

        assert_eq!(confs, config_new);
        // }

        // #[test]
        // fn test_set_read_env() {
        //     init();
        env_cleaner();
        std::env::set_var("GROUP_ID", "13");
        std::env::set_var("base_url", "https://test.tst.ts/user");

        let confs = load_config().unwrap();

        assert_eq!("13".to_string(), confs.group_id.unwrap().to_string());
        assert_eq!(
            "https://test.tst.ts/user".to_string(),
            confs.base_url.unwrap()
        );
        // }

        // #[test]
        // fn test_env_and_file() {
        //     init();
        env_cleaner();

        std::env::set_var("GROUP_ID", "13");
        std::env::set_var("ENV_FILE", ".env.example");

        let confs = load_config().unwrap();
        assert_eq!("13".to_string(), confs.group_id.unwrap().to_string());
        assert_eq!("mail.com".to_string(), confs.smtp_server.unwrap());
        // }

        // #[test]
        // fn test_no_file() {
        // init();
        env_cleaner();

        std::env::set_var("GROUP_ID", "13");
        std::env::set_var("ENV_FILE", ".env.null");

        let confs = load_config().unwrap();
        assert_eq!("13".to_string(), confs.group_id.unwrap().to_string());
    }
}
