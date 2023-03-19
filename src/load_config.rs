use log::{debug, error, info, warn};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::{collections::HashMap, fmt::Error};

fn read_lines(filename: String) -> Option<io::Lines<BufReader<File>>> {
    // Open the file in read-only mode.
    let file = File::open(&filename);
    // Read the file line by line, and return an iterator of the lines of the file.

    match file {
        Ok(content) => Some(io::BufReader::new(content).lines()),
        Err(_) => {
            error!("Couldn't read the file {}", filename);
            None
        }
    }
}

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

    #[test]
    fn test_read_file() {
        init();

        let lines = read_lines(".env.example".to_string()).unwrap();

        for line in lines {
            println!("{:?}", line.unwrap());
        }
    }
}
