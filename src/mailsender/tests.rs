#[cfg(test)]
mod test_mail {
    use crate::load_config;

    // use super::*;

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
    fn test_get_server_port() {
        init();

        std::env::set_var("SMTP_SERVER", " mail.server.com:25 ");

        let config = load_config().unwrap();

        let binding = config.smtp.unwrap().server.unwrap();
        println!("{:?}", binding.contains(':'));
        let test:Vec<_> = binding.trim().split(':').collect();

        println!("{:?}", test);

    }

    #[test]
    #[ignore = "needs input"]
    fn test_validade_mail() {
        init();

        let mut config = load_config().unwrap();

        let mut passw = String::new();

        if config.smtp.is_some() && config.smtp.clone().unwrap().pass.is_none() {
            print!("give a password for mail test: ");
            if std::io::stdin().read_line(&mut passw).is_ok() {
                println!("\nPassword received.");
            }
        }

        config.smtp.as_mut().unwrap().pass = Some(passw);

        assert!(config.smtp.as_ref().unwrap().is_valid(), "Configuration isn't valid: {:?}", config)
    }

    #[test]
    #[ignore = "needs password"]
    fn test_send_message() {
        init();
        let config = load_config().unwrap();

        match config.smtp.unwrap().send_plain_text("Test message".to_string()) {
            Ok(_) => assert!(true),
            Err(e) => panic!("Error while sending email: {:?}", e),
        }
    }
}
