use std::collections::HashMap;
use std::env;

/// Get configurations from environment or from file
fn get_configs() -> HashMap<&'static str, String>{

    /// Possible configuration keys
    let config_keys = (
        "GROUP_ID",
        "PROJECT_ID",
        "PRIVATE_TOKEN",
        "BASE_URL",
        "SMTP_SERVER",
        "SMTP_USER",
        "SMTP_FROM",
        "SMTP_TO",
        "SMTP_SUBJECT",
    );



    HashMap::from([(config_keys.1, "teste".to_string())])

}