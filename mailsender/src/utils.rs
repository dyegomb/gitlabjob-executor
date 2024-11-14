use crate::prelude::*;

pub trait SmtpUtils {
    fn split_server_port(server_with_port: String) -> Option<(String, u16)> {
        let port: u16;
        let server: String;

        let vector = server_with_port.trim().split(':').collect::<Vec<_>>();

        if vector.len() == 2 {
            if let Ok(port_num) = vector[1].parse::<u16>() {
                port = port_num;
                server = vector[0].to_owned();
            } else {
                return None;
            }
        } else if vector.len() == 1 {
            port = DEFAULT_SMTP_PORT;
            server = vector[0].to_owned();
        } else {
            return None;
        }

        Some((server, port))
    }

    // Yeah, I liked this pun
    fn body_builder(
        &self,
        subject: String,
        message: String,
        destination: Option<String>,
    ) -> Message;
}

impl SmtpUtils for SmtpConfig {
    fn body_builder(
        &self,
        subject: String,
        message: String,
        destination: Option<String>,
    ) -> Message {
        if !self.is_valid() {
            error!("Smtp configuration is invalid");
            std::process::exit(31)
        };

        let concat_subject = format!(
            "{}{}",
            self.subject.clone().unwrap_or("".to_owned()),
            subject
        );

        let string_to = match destination {
            Some(dest) => {
                format!(
                    "{}, {}",
                    self.to.clone().unwrap_or("".to_owned()),
                    dest.as_str()
                )
            }
            None => self.to.clone().unwrap(),
        };

        let to: Mailboxes = match string_to.parse() {
            Ok(dest) => dest,
            Err(_) => self.to.as_ref().unwrap().parse().unwrap(),
        };

        let to_header: lettre::message::header::To = to.into();
        debug!("Mail recipients: {:?}", to_header);

        // match Message::builder()
        match MessageBuilder::new()
            .mailbox(to_header)
            .from(self.from.as_ref().unwrap().parse().unwrap())
            // .reply_to(.parse().unwrap())
            // .to(self.to.as_ref().unwrap().parse().unwrap())
            // .to(to)
            .subject(concat_subject)
            .header(ContentType::TEXT_HTML)
            .body(message)
        {
            Ok(message) => message,
            Err(_) => {
                error!("Couldn't build a mail message");
                std::process::exit(32)
            }
        }
    }
}
