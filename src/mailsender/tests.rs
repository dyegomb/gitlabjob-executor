#[cfg(test)]
mod test_mail {
    use crate::load_config;
    use crate::mailsender::*;
    // use crate::mailsender::utils::SmtpUtils;

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
    fn test_lettre_only() {
        init();

        let config = load_config().unwrap();

        let to = config.smtp.clone().unwrap().to.unwrap();
        let from = config.smtp.clone().unwrap().from.unwrap();
        let server = config.smtp.clone().unwrap().server.unwrap();
        let user = config.smtp.clone().unwrap().user.unwrap();
        let pass = config.smtp.clone().unwrap().pass.unwrap();
        let subject = config.smtp.unwrap().subject.unwrap();

        use lettre::message::header::ContentType;
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{Message, SmtpTransport, Transport};

        let email = Message::builder()
            .from(from.parse().unwrap())
            .reply_to(from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(String::from("Be happy!"))
            .unwrap();

        let creds = Credentials::new(user, pass);

        // Open a remote connection to gmail
        let mailer = SmtpTransport::starttls_relay(&server)
            .unwrap()
            .port(25)
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => panic!("Could not send email: {:?}", e),
        }
    }

    #[test]
    fn test_invalid_cert() {
        use lettre::{
            message::header::ContentType,
            transport::smtp::{
                authentication::Credentials,
                client::{Tls, TlsParameters},
            },
            Message, SmtpTransport, Transport,
        };

        init();

        let config = load_config().unwrap();

        let to = config.smtp.clone().unwrap().to.unwrap();
        let from = config.smtp.clone().unwrap().from.unwrap();
        let server = config.smtp.clone().unwrap().server.unwrap();
        let user = config.smtp.clone().unwrap().user.unwrap();
        let pass = config.smtp.clone().unwrap().pass.unwrap();
        let subject = config.smtp.unwrap().subject.unwrap();

        let email = Message::builder()
            .from(from.parse().unwrap())
            .reply_to(from.parse().unwrap())
            .to(to.parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(String::from("Be happy!"))
            .unwrap();

        let creds = Credentials::new(user, pass);

        let tls = TlsParameters::builder(server.to_owned())
            // .dangerous_accept_invalid_hostnames(true)
            .dangerous_accept_invalid_certs(true)
            // .set_min_tls_version(lettre::transport::smtp::client::TlsVersion::Tlsv10)
            .build()
            .unwrap();

        // let mailer = SmtpTransport::builder_dangerous(server)
        let mailer = SmtpTransport::relay(&server)
            .unwrap()
            .port(25)
            .tls(Tls::Opportunistic(tls))
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => println!("Email sent successfully!"),
            Err(e) => panic!("Could not send email: {e:?}"),
        }
    }

    #[test]
    fn test_get_server_port() {
        init();

        std::env::set_var("SMTP_SERVER", " mail.server.com:123 ");

        let config = load_config().unwrap();

        let binding = config.smtp.unwrap().server.unwrap();

        let test = Smtp::split_server_port(binding).unwrap();

        assert_eq!(test, ("mail.server.com".to_owned(), 123_u32))
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

        assert!(
            config.smtp.as_ref().unwrap().is_valid(),
            "Configuration isn't valid: {:?}",
            config
        )
    }

    #[test]
    #[ignore = "needs password"]
    fn test_send_message() {
        init();
        let config = load_config().unwrap();

        match config
            .smtp
            .unwrap()
            .send_plain_text("Test message".to_string())
        {
            Ok(_) => assert!(true),
            Err(e) => panic!("Error while sending email: {:?}", e),
        }
    }
}
