// use std::str::FromStr;

// use std::fmt::Error;

use std::str::FromStr;

// use lettre::message::header::ContentType;
use lettre::transport::smtp;
// use lettre::transport::smtp::authentication::Credentials;
use lettre::Transport;

extern crate email_address;
extern crate lettre;

use log::error;
use merge::Merge;
use serde::Deserialize;

mod tests;
mod utils;

#[derive(Debug)]
pub enum MailError {
    ValidationError,
    SendMailError,
}

#[derive(Deserialize, Debug, Merge, PartialEq, Clone)]
pub struct Smtp {
    pub server: Option<String>,
    port: Option<usize>,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
}

impl Smtp {
    pub fn default() -> Self {
        Smtp {
            server: None,
            user: None,
            pass: None,
            from: None,
            to: None,
            subject: None,
            port: Some(25)
        }
    }

    // https://github.com/lettre/lettre/blob/master/examples/smtp_selfsigned.rs
    // https://github.com/lettre/lettre/blob/master/examples/autoconfigure.rs



    pub fn is_valid(&self) -> bool {
        match &self.to {
            None => false,
            Some(to) => match &self.from {
                None => false,
                Some(from) => {
                    email_address::EmailAddress::is_valid(from)
                        && email_address::EmailAddress::is_valid(to)
                }
            },
        };

        self.server.is_some()
            && self.subject.is_some()
            && ((self.user.is_some() && self.pass.is_some())
                || (self.user.is_none() && self.pass.is_none()))
    }

    pub fn send_plain_text(&self, text: String) -> Result<&'static str, MailError> {
        if !self.is_valid() {
            return Err(MailError::ValidationError);
        }

        let from =
            lettre::message::Mailbox::from_str(self.from.as_ref().unwrap().as_str()).unwrap();
        let to = lettre::message::Mailbox::from_str(self.to.as_ref().unwrap().as_str()).unwrap();

        let email = lettre::Message::builder()
            .from(from)
            .to(to)
            .subject(self.clone().subject.unwrap())
            .header(lettre::message::header::ContentType::TEXT_PLAIN)
            .body(text)
            .unwrap();

        let mailer;

        if self.user.is_some() {
            let creds = smtp::authentication::Credentials::new(
                self.user.clone().unwrap(),
                self.pass.clone().unwrap(),
            );

            mailer = lettre::SmtpTransport::relay(self.server.as_ref().unwrap())
                .unwrap()
                .credentials(creds)
                .port(25)
                // .tls(tls)
                .build();
        } else {
            mailer = lettre::SmtpTransport::relay(self.server.as_ref().unwrap())
                .unwrap()
                .port(25)
                .build();
        };

        match mailer.send(&email) {
            Ok(_) => Ok("Email sent successfully"),
            Err(e) => {
                error!("Error while sending email: {}", e);
                Err(MailError::SendMailError)
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
    }
}

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
