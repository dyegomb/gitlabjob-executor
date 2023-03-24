use log::{debug, error};
use merge::Merge;
use serde::Deserialize;


use crate::mail_sender::Smtp;

extern crate envy;
extern crate merge;
extern crate toml;

/// Possible configurations
#[derive(Deserialize, Debug, Merge, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Config {
    pub group_id: Option<usize>,
    pub project_id: Option<usize>,
    pub private_token: Option<String>,
    pub base_url: Option<String>,
    pub smtp: Option<Smtp>,
}

#[derive(Debug)]
pub enum ConfigErrors {
    EnvironError,
    FileError,
}

/// Get configurations from environment or from file
pub fn load_config() -> Result<Config, ConfigErrors> {
    let mut config;

    // Load config from environment variables
    match envy::from_env::<Config>() {
        Ok(env_config) => {
            config = env_config;
        }
        Err(err) => {
            error!("Error while reading environment variables: {:?}", err);
            return Err(ConfigErrors::EnvironError);
        }
    };

    // SMTP settings from environment variables
    if std::env::vars().any(|(k, _)| k.starts_with("SMTP_")) {
        let mut smtp_config = Smtp::new();

        std::env::vars()
            .filter(|(k, _)| k.starts_with("SMTP_"))
            .for_each(|(k, v)| match k.as_str() {
                "SMTP_USER" => smtp_config.user = Some(v),
                "SMTP_SERVER" => smtp_config.server = Some(v),
                "SMTP_PASS" => smtp_config.pass = Some(v),
                "SMTP_FROM" => smtp_config.from = Some(v),
                "SMTP_TO" => smtp_config.to = Some(v),
                "SMTP_SUBJECT" => smtp_config.subject = Some(v),
                _ => {}
            });

        config.smtp = Some(smtp_config);
    }

    let env_file = std::env::var("ENV_FILE").unwrap_or(".env".to_string());

    if let Ok(content) = std::fs::read_to_string(&env_file) {
        debug!("Reading {} file.", &env_file);

        match toml::from_str::<Config>(&content) {
            Ok(toml_text) => {
                let config_file: Config = toml_text;
                config.merge(config_file);
            }
            Err(err) => {
                error!("Couldn't read file {}: {}", env_file, err);
                return Err(ConfigErrors::FileError);
            }
        };
    };

    debug!("{:?}", config);
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
    #[ignore = "concurrency"]
    fn test_empty() {
        // running all tests in same place cause concurrency problems
        init();
        env_cleaner();

        std::env::set_var("ENV_FILE", ".env.none");

        let confs = load_config().unwrap();
        let config_new = Config {
            group_id: None,
            project_id: None,
            private_token: None,
            base_url: None,
            smtp: None,
        };

        assert_eq!(confs, config_new);
    }

    #[test]
    #[ignore = "concurrency"]
    fn test_set_read_env() {
        init();
        env_cleaner();
        std::env::set_var("GROUP_ID", "13");
        std::env::set_var("base_url", "https://test.tst.ts/user");

        let confs = load_config().unwrap();

        assert_eq!("13".to_string(), confs.group_id.unwrap().to_string());
        assert_eq!(
            "https://test.tst.ts/user".to_string(),
            confs.base_url.unwrap()
        );
    }

    #[test]
    #[ignore = "concurrency"]
    fn test_env_and_file() {
        init();
        env_cleaner();

        std::env::set_var("GROUP_ID", "13");
        std::env::set_var("ENV_FILE", ".env.example");

        let confs = load_config().unwrap();
        assert_eq!("13".to_string(), confs.group_id.unwrap().to_string());
        assert_eq!("mail.com".to_string(), confs.smtp.unwrap().server.unwrap());
    }

    #[test]
    #[ignore = "concurrency"]
    fn test_no_file() {
        init();
        env_cleaner();

        std::env::set_var("GROUP_ID", "13");
        std::env::set_var("ENV_FILE", ".env.null");
        std::env::set_var("SMTP_USER", "user.mail");
        std::env::set_var("SMTP_PASS", "$ecRet@#");

        let confs = load_config().unwrap();
        assert_eq!("13".to_string(), confs.group_id.unwrap().to_string());
        assert_eq!("$ecRet@#", &confs.smtp.clone().unwrap().pass.unwrap());
        assert_eq!("user.mail", &confs.smtp.unwrap().user.unwrap());
    }
}
