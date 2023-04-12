// extern crate email_address;
extern crate lettre;

use merge::Merge;
use serde::Deserialize;

mod tests;
mod utils;

use utils::SmtpUtils;

// https://github.com/lettre/lettre/blob/master/examples/smtp_selfsigned.rs
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::{
        authentication::Credentials,
        client::{Certificate, Tls, TlsParameters},
    },
    Message, SmtpTransport, Transport,
};

const DEFAULT_SMTP_PORT: u32 = 25;

#[derive(Deserialize, Debug, Merge, PartialEq, Clone)]
pub struct Smtp {
    pub server: Option<String>,
    port: Option<u32>,
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
            port: Some(DEFAULT_SMTP_PORT),
        }
    }

    // https://github.com/lettre/lettre/blob/master/examples/smtp_selfsigned.rs
    // https://github.com/lettre/lettre/blob/master/examples/autoconfigure.rs

    pub fn is_valid(&self) -> bool {
        let addresses = match &self.to {
            None => false,
            Some(to) => match &self.from {
                None => false,
                Some(from) => to.parse::<Mailbox>().is_ok() && from.parse::<Mailbox>().is_ok(),
            },
        };

        addresses
            && self.server.is_some()
            && self.subject.is_some()
            && ((self.user.is_some() && self.pass.is_some())
                || (self.user.is_none() && self.pass.is_none()))
    }

    pub fn send_plain_text(&self, text: String) -> Result<(), String> {
        if !self.is_valid() {
            return Err("Erro".to_owned());
        }
        todo!()
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
