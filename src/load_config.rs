use log::{debug, warn};
use std::{collections::HashMap, fmt::Error};

/// Get configurations from environment or from file
pub fn load_config() -> Result<HashMap<&'static str, String>, Error> {
    // Possible configuration keys
    let config_keys = vec![
        "GROUP_ID",
        "PROJECT_ID",
        "PRIVATE_TOKEN",
        "BASE_URL",
        "SMTP_SERVER",
        "SMTP_USER",
        "SMTP_FROM",
        "SMTP_TO",
        "SMTP_SUBJECT",
    ];

    let mut configs = HashMap::new();

    let mut all_read = true;

    #[cfg(debug_assertions)]
    let mut not_setted: Vec<&str> = Vec::new();

    config_keys.iter().for_each(|k| {
        if let Ok(val) = std::env::var(k) {
            configs.insert(*k, val);
        } else {
            all_read = false;

            #[cfg(debug_assertions)]
            not_setted.push(*k);
        }
    });

    let file_name = std::env::var("ENV_FILE").unwrap_or(".env".to_string());

    // let file_content = std::fs::read(file_name)
    //     .expect("Should have been able to read the file");

    if !all_read {
        warn!("Not all settings were setted");

        #[cfg(debug_assertions)]
        debug!("Not setted: {:?}", not_setted);
    }

    Ok(configs)
    //HashMap::from([(config_keys[0], "teste")])
}

#[cfg(test)]
mod test_load_config {
    use log::info;

    use super::*;

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
    fn test_set_read_env() {
        init();
        std::env::set_var("GROUP_ID", "Test_val");

        let confs = load_config().unwrap();

        warn!("Warning");
        debug!("Debugging");
        info!("Info");
        println!("Printing");

        assert_eq!("Test_val", confs.get("GROUP_ID").unwrap());
    }

    #[test]
    fn test_all_settings() {
        init();
        let config_keys = vec![
            "GROUP_ID",
            "PROJECT_ID",
            "PRIVATE_TOKEN",
            "BASE_URL",
            "SMTP_SERVER",
            "SMTP_USER",
            "SMTP_FROM",
            "SMTP_TO",
            "SMTP_SUBJECT",
        ];

        let common_val = vec!["Test_val".to_string(); config_keys.len()];

        config_keys
            .iter()
            .zip(&common_val)
            .for_each(|(k, v)| std::env::set_var(k, v));

        let hash_templ: HashMap<_, _> =
            HashMap::from_iter(config_keys.iter().zip(common_val).map(|(k, v)| (*k, v)));

        let confs = load_config().unwrap();

        assert_eq!(hash_templ, confs);
    }
}
