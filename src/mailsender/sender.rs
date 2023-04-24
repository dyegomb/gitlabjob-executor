use crate::mailsender::prelude::*;
use tokio::time::Duration;

#[derive(Clone)]
pub struct MailSender {
    pub relay: Option<SmtpTransport>,
    server: String,
    port: u16,
    user: Option<String>,
    pass: Option<String>,
}


/// Implements a mail sender based on lettre
impl MailSender {
    pub async fn try_new(smtp_config: SmtpConfig) -> Result<Self, String> {
        if !smtp_config.is_valid() {
            return Err("Invalid smtp configurations".to_owned());
        }

        let error = { Err("Unable to get smtp server".to_owned()) };

        let (server, port) = match &smtp_config.server {
            Some(full_server) => match SmtpConfig::split_server_port(full_server.to_owned()) {
                Some((server_name, port)) => (server_name, port),
                None => error?,
            },
            None => error?,
        };

        let user = smtp_config.user.clone();
        let pass = smtp_config.pass;

        let mut mailer = Self {
            server,
            port,
            user,
            pass,
            relay: None,
        };

        match mailer.try_build_relay() {
            Ok(_) => Ok(mailer),
            Err(error) => Err(error),
        }
    }

    /// Try to autoconfigure mail sender
    /// based on: <https://github.com/lettre/lettre/blob/master/examples/autoconfigure.rs>
    fn try_build_relay(&mut self) -> Result<(), String> {

        let wait_time = Some(Duration::from_secs(20));

        let creds = if self.user.is_some() && self.pass.is_some() {
            Some(Credentials::new(
                self.user.as_ref().unwrap().to_owned(),
                self.pass.as_ref().unwrap().to_owned(),
            ))
        } else {
            warn!("Proceeding with unauthenticated smtp connection");
            None
        };


        let mut mailer = if creds.is_some() {
            match SmtpTransport::relay(&self.server) {
                Ok(relay) => relay.credentials(creds.unwrap()).port(self.port),
                Err(_) => return Err("Couldn't build mailer".to_owned()),
            }
        } else {
            match SmtpTransport::relay(&self.server) {
                Ok(relay) => relay.port(self.port),
                Err(_) => return Err("Couldn't build mailer".to_owned()),
            }
        };

        // First try: Smtp over TLS
        match mailer.clone().timeout(wait_time).build().test_connection() {
            Ok(_) => {
                self.relay = Some(mailer.build());
                return Ok(());
            }
            Err(err) => debug!("First try to build mailer didn't work: {err}"),
        }

        // Second try: Stmp with STARTTLS
        let tls_builder = TlsParameters::builder(self.server.to_owned());

        let mut tls = tls_builder.clone().build().expect("Error while building TLS support");
        mailer = mailer.tls(Tls::Opportunistic(tls));

        match mailer.clone().timeout(wait_time).build().test_connection() {
            Ok(_) => {
                self.relay = Some(mailer.build());
                return Ok(());
            }
            Err(err) => debug!("Second try to build mailer didn't work: {}", err),
        }

        // Third try: Smtp with STARTTLS with invalid certificate
        tls = tls_builder.dangerous_accept_invalid_certs(true).build().expect("Error while building TLS support");

        mailer = mailer.tls(Tls::Opportunistic(tls));

        match mailer.clone().timeout(wait_time).build().test_connection() {
            Ok(_) => {
                warn!("Smtp server with invalid certificate");
                self.relay = Some(mailer.build()) ;
                return Ok(());
            }
            Err(err) => debug!("Third try to build mailer didn't work: {}", err),
        }

        // Fourth try: WITHOUT ENCRIPTION!
        mailer = mailer.tls(Tls::None);

        match mailer.clone().timeout(wait_time).build().test_connection() {
            Ok(_) => {
                warn!("!!! SMTP CONNECTION WITHOUT ENCRYPTION !!!");
                self.relay = Some(mailer.build());
                Ok(())
            },
            Err(err) => {
                error!("Couldn't build mailer: {err}");
                Err("Couldn't build mailer".to_owned())
            },
        }
    }
}
