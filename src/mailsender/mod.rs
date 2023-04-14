mod tests;
mod utils;
mod smtpconfig;
mod sender;

pub mod prelude {
    pub const DEFAULT_SMTP_PORT: u16 = 587;

    pub use log::{debug, warn, error};
    pub use merge::Merge;
    pub use serde::Deserialize;
    pub use lettre::{Message, SmtpTransport, Transport};
    pub use lettre::message::header::ContentType;
    pub use lettre::message::Mailbox;
    pub use lettre::transport::smtp::authentication::Credentials;
    pub use lettre::transport::smtp::client::{Tls, TlsParameters};

    pub use crate::mailsender::smtpconfig::SmtpConfig;
    pub use crate::load_config::Config;
    pub use super::sender::MailSender;
    pub use super::utils::SmtpUtils;
}
