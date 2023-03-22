use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use merge::Merge;
use serde::Deserialize;

#[derive(Deserialize, Debug, Merge, PartialEq, Clone)]
pub struct SMTP {
    pub server: Option<String>,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
}

impl SMTP {
    pub fn new() -> Self {
        SMTP {
            server: None,
            user: None,
            pass: None,
            from: None,
            to: None,
            subject: None,
        }
    }
    pub fn is_valid(self) -> bool {
        true
    }
}

// let email = Message::builder()
//     .from("NoBody <nobody@domain.tld>".parse().unwrap())
//     .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
//     .to("Hei <hei@domain.tld>".parse().unwrap())
//     .subject("Happy new year")
//     .header(ContentType::TEXT_PLAIN)
//     .body(String::from("Be happy!"))
//     .unwrap();

// let creds = Credentials::new("smtp_username".to_owned(), "smtp_password".to_owned());

// // Open a remote connection to gmail
// let mailer = SmtpTransport::relay("smtp.gmail.com")
//     .unwrap()
//     .credentials(creds)
//     .build();

// // Send the email
// match mailer.send(&email) {
//     Ok(_) => println!("Email sent successfully!"),
//     Err(e) => panic!("Could not send email: {:?}", e),
// }
#[cfg(test)]
mod test_mail {
    use super::*;

    #[test]
    #[ignore = "needs input"]
    fn test_create_mail() {
        use crate::load_config;

        let config = load_config().unwrap();

        let mut passw = String::new();

        let mut get_pass = || {
            print!("give a password for mail test: ");

            if std::io::stdin().read_line(&mut passw).is_ok() {
                println!("\nPassword received.");
            }
        };
        if config.smtp.is_some() {
            if config.smtp.unwrap().pass.is_none() {
                get_pass();
            }
        } else if config.smtp.unwrap().pass.is_none() {
            get_pass();
        }
    }
}
