use crate::mailsender::{MailSender, SmtpConfig, DEFAULT_SMTP_PORT, Message};

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
    fn body_builder(&self, message: String) -> Message;
}

impl SmtpUtils for SmtpConfig {
    fn body_builder(&self, message: String) -> Message {
       todo!() 
    }
}
