mod sender;
// mod smtpconfig;
mod tests;
mod utils;

pub mod prelude {
    pub const DEFAULT_SMTP_PORT: u16 = 587;

    pub use lettre::message::header::ContentType;
    pub use lettre::message::{Mailboxes, MessageBuilder};
    pub use lettre::transport::smtp::authentication::Credentials;
    pub use lettre::transport::smtp::client::{Tls, TlsParameters};
    pub use lettre::{Message, SmtpTransport, Transport};
    pub use log::{debug, error, warn};
    pub use merge::Merge;
    pub use serde::Deserialize;

    pub use super::sender::MailSender;
    pub use configloader::SmtpConfig;
    // pub use super::smtpconfig::SmtpConfig;
    pub use super::utils::SmtpUtils;
}
