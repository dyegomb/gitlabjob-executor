

use crate::mailsender::prelude::*;

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
    fn body_builder(&self, subject: String, message: String) -> Message;
}

impl SmtpUtils for SmtpConfig {
    fn body_builder(&self, subject: String, message: String) -> Message {
        if !self.is_valid() {
            panic!("Smtp configuration is invalid")
        };

        let concat_subject = format!("{}{}", self.subject.clone().unwrap_or("".to_owned()), subject);

        match Message::builder()
        .from(self.from.as_ref().unwrap().parse().unwrap())
        // .reply_to(.parse().unwrap())
        .to(self.to.as_ref().unwrap().parse().unwrap())
        .subject(concat_subject)
        .header(ContentType::TEXT_HTML)
        .body(message) {
            Ok(message) => message,
            Err(_) => panic!("Couldn't build a mail message"),
        }
    }
}
