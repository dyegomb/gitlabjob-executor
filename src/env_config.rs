use std::collections::HashMap;

/// Get configurations from environment or from file
fn get_configs() -> HashMap<&'static str, &'static str>{

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

    config_keys.iter()
        .for_each(|k| {
            if let Ok(val) = std::env::var(k) {
                configs.insert(k, val);
            }
        });

    let env_file = std::env::var("ENV_FILE").unwrap_or(".env".to_string());

    HashMap::from([(config_keys[0], "teste")])

}