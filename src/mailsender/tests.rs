#[cfg(test)]
mod test_mail {
    use log::debug;

    use crate::configloader::prelude::*;
    use crate::mailsender::prelude::*;

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

        std::env::set_var("SMTP_SERVER", " mail.server.com:123 ");

        let config = Config::load_config().unwrap();

        let binding = config.smtp.unwrap().server.unwrap();

        let test = SmtpConfig::split_server_port(binding).unwrap();

        assert_eq!(test, ("mail.server.com".to_owned(), 123_u16))
    }

    #[test]
    #[ignore = "needs input"]
    fn test_validade_mail() {
        init();

        let mut config = Config::load_config().unwrap();

        let mut passw = String::new();

        if config.smtp.is_some() && config.smtp.clone().unwrap().pass.is_none() {
            print!("give a password for mail test: ");
            if std::io::stdin().read_line(&mut passw).is_ok() {
                println!("\nPassword received.");
            }
        }

        config.smtp.as_mut().unwrap().pass = Some(passw);

        assert!(
            config.smtp.as_ref().unwrap().is_valid(),
            "Configuration isn't valid: {:?}",
            config
        )
    }

    #[tokio::test]
    async fn test_try_new() {
        init();

        let config = Config::load_config().unwrap();

        let mailer_build = MailSender::try_new(config.smtp.unwrap());

        match mailer_build.await {
            Ok(_) => debug!("New mailer has been built."),
            Err(error) => {
                panic!("{}", error)
            }
        }
    }

    #[test]
    fn test_build_mail_message() {
        init();

        let config = Config::load_config().unwrap();

        let message = r#"
This is a <b>test message</b>. :-)
"#;

        let mail_message = config
            .smtp
            .unwrap()
            .body_builder("Test subject".to_owned(), message.to_owned());

        debug!("{:?}", mail_message);
    }

    #[tokio::test]
    #[ignore = "It'll really send an email message"]
    async fn test_send_mail() {
        init();

        let config = Config::load_config().unwrap();

        let smtp_config = config.smtp.clone();

        let message = r#"
<h1>Test</h1>
This is a <b>test message</b>. :-)
"#;

        let mail_message = config
            .smtp
            .clone()
            .unwrap()
            .body_builder("Test subject".to_owned(), message.to_owned());

        let mail_message2 = config
            .smtp
            .unwrap()
            .body_builder("Test subject".to_owned(), "Another message test".to_owned());

        let mailsender = MailSender::try_new(smtp_config.unwrap()).await.unwrap();


        if let Some(relay) =  mailsender.relay {
                match relay.send(&mail_message) {
                    Ok(_) => debug!("Message 1 sent"),
                    Err(err) => panic!("{}", err)
                }; 
                match relay.send(&mail_message2) {
                    Ok(_) => debug!("Message 2 sent"),
                    Err(err) => panic!("{}", err)
                };
        };
    }
}
