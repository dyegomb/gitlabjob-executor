use merge::Merge;
use serde::Deserialize;
use lettre::message::Mailboxes;

// use crate::prelude::*;

/// Configurations to build mail report function
#[derive(Deserialize, Default, Debug, Merge, PartialEq, Clone)]
pub struct SmtpConfig {
    pub server: Option<String>,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub subject: Option<String>,
}

impl SmtpConfig {

    /// Validates  head mail fields
    pub fn is_valid(&self) -> bool {
        let addresses = match &self.to {
            None => false,
            Some(to) => match &self.from {
                None => false,
                Some(from) => to.parse::<Mailboxes>().is_ok() && from.parse::<Mailboxes>().is_ok(),
            },
        };

        addresses
            && self.server.is_some()
            && self.subject.is_some()
            && ((self.user.is_some() && self.pass.is_some())
                || (self.user.is_none() && self.pass.is_none()))
    }
}
