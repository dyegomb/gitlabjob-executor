use mailsender::prelude::{MailSender, SmtpTransport};

use crate::Config;
use log::error;

pub async fn mailrelay_buid(config: &Config) -> Option<SmtpTransport> {
    match &config.smtp {
        Some(smtp) => match smtp.is_valid() {
            true => match MailSender::try_new(smtp.clone()).await {
                Ok(mailer) => mailer.relay,
                Err(error) => {
                    error!("{}", error);
                    None
                }
            },
            false => None,
        },
        None => None,
    }
}

// pub fn adjust