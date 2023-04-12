// extern crate email_address;
extern crate lettre;

mod tests;
mod utils;

use utils::SmtpUtils;

use merge::Merge;
use serde::Deserialize;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::{
        authentication::Credentials,
        client::{Certificate, Tls, TlsParameters},
    },
    Message, SmtpTransport, Transport,
};

const DEFAULT_SMTP_PORT: u32 = 587;

#[derive(Deserialize, Debug, Merge, PartialEq, Clone)]
pub struct Smtp {
    pub server: Option<String>,
    pub user: Option<String>,
    pass: Option<String>,
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
        }
    }

    pub fn is_valid(&self) -> bool {
        let addresses = match &self.to {
            None => false,
            Some(to) => match &self.from {
                None => false,
                Some(from) => {
                    to.parse::<Mailbox>().is_ok() && from.parse::<Mailbox>().is_ok()
                    }
                }
            };

        addresses
            && self.server.is_some()
            && self.subject.is_some()
            && ((self.user.is_some() && self.pass.is_some())
                || (self.user.is_none() && self.pass.is_none()))
    }
}

pub struct MailSender {
    relay: Option<SmtpTransport>,
    server: String,
    port: u32,
    user: Option<String>,
    pass: Option<String>
}

impl MailSender {
    pub fn try_new(smtp_config: &Smtp) -> Result<Self, String> {
        if !smtp_config.is_valid() {
            return Err("Invalid smtp configurations".to_owned());
        }
        let error = { Err("Unable to get smtp server".to_owned()) };

        let (server, port) = match &smtp_config.server {
            Some(full_server) => match Self::split_server_port(full_server.to_owned()) {
                Some((server_name, port),) => (server_name, port),
                // None => { return Err("Unable to collect smtp server".to_owned()) }
                None => { error? }
            },
            None => error?,
        };

        let user = smtp_config.user;
        let pass = smtp_config.pass; 

        Ok(Self{relay: None, server, port, user, pass})
    }

    fn try_build_relay(&mut self) -> Result<(), String> {
        // https://github.com/lettre/lettre/blob/master/examples/autoconfigure.rs


        // let creds = Credentials::new(user, pass);

        // let tls = TlsParameters::builder(server.to_owned())
        //     // .dangerous_accept_invalid_hostnames(true)
        //     .dangerous_accept_invalid_certs(true)
        //     // .set_min_tls_version(lettre::transport::smtp::client::TlsVersion::Tlsv10)
        //     .build()
        //     .unwrap();

        // // let mailer = SmtpTransport::builder_dangerous(server)
        // let mailer = SmtpTransport::relay(&server)
        //     .unwrap()
        //     .port(25)
        //     .tls(Tls::Opportunistic(tls))
        //     .credentials(creds)
        //     .build();




        todo!()
    }
}