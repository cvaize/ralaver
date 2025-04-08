use crate::config::MailSmtpConfig;
use crate::Config;
use actix_web::web::Data;
use lettre::error::Error as LettreError;
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{
    Address as LettreAddress, Message as LettreMessage, SmtpTransport as LettreSmtpTransport,
    Transport as LettreTransport,
};

pub struct MailService {
    config: Data<Config>,
    mailer: Option<LettreSmtpTransport>,
}

impl MailService {
    pub fn new(config: Data<Config>, mailer: Option<LettreSmtpTransport>) -> Self {
        Self { config, mailer }
    }

    pub fn send_email(&mut self, data: &EmailMessage) -> Result<(), MailServiceError> {
        if self.mailer.is_none() {
            self.connect()?;
        }

        let mailer = self.mailer.as_ref().unwrap();
        let message: LettreMessage = data.build(&self.config.mail.smtp)?;

        mailer.send(&message).map_err(|_| MailServiceError::SendFail)?;

        Ok(())
    }

    pub fn connect(&mut self) -> Result<(), MailServiceError> {
        let config = self.config.get_ref();
        let host = config.mail.smtp.host.to_owned();
        let port: u16 = config.mail.smtp.port.to_owned().parse()
            .map_err(|_| MailServiceError::ConnectSmtpFail)?;
        let username = config.mail.smtp.username.to_owned();
        let password = config.mail.smtp.password.to_owned();

        let creds = Credentials::new(username, password);

        let mut mailer = LettreSmtpTransport::builder_dangerous(host.to_owned())
            .port(port);

        if config.mail.smtp.encryption == "" {
        } else if config.mail.smtp.encryption == "tls" {
            let tls_parameters = TlsParameters::new(host.into())
                .map_err(|_| MailServiceError::ConnectSmtpFail)?;

            mailer = mailer.tls(Tls::Wrapper(tls_parameters));
        } else {
            return Err(MailServiceError::EncryptionNotSupported);
        }

        let mailer = mailer.credentials(creds).build();

        self.mailer = Some(mailer);

        Ok(())
    }
}

#[derive(Debug)]
pub struct EmailMessage {
    pub from: Option<EmailAddress>,
    pub reply_to: Option<EmailAddress>,
    pub to: EmailAddress,
    pub subject: String,
    pub html_body: Option<String>,
    pub text_body: String,
}

#[derive(Debug)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub email: String,
}

#[derive(Debug, Clone, Copy)]
pub enum MailServiceError {
    SendFail,
    BuildMessageFromAddressFail,
    BuildMessageReplyToAddressFail,
    BuildMessageToAddressFail,
    BuildMessageBodyFail,
    BuildMessageFail,
    ConnectSmtpFail,
    EncryptionNotSupported,
}

impl EmailMessage {
    fn build(&self, smtp: &MailSmtpConfig) -> Result<LettreMessage, MailServiceError> {
        let mut builder = LettreMessage::builder();

        if let Some(from) = &self.from {
            builder = builder.from(Mailbox::new(
                from.name.to_owned(),
                from.email
                    .parse::<LettreAddress>()
                    .map_err(|_| MailServiceError::BuildMessageFromAddressFail)?,
            ));
        } else {
            let mut from_name: Option<String> = None;
            let mut from_address: Option<String> = None;

            if smtp.from_name != "" {
                from_name = Some(smtp.from_name.to_owned());
            }

            if smtp.from_address != "" {
                from_address = Some(smtp.from_address.to_owned());
            }

            if let Some(from_address) = from_address {
                builder = builder.from(Mailbox::new(
                    from_name,
                    from_address
                        .parse::<LettreAddress>()
                        .map_err(|_| MailServiceError::BuildMessageFromAddressFail)?,
                ));
            }
        }

        if let Some(reply_to) = &self.reply_to {
            builder = builder.reply_to(Mailbox::new(
                reply_to.name.to_owned(),
                reply_to
                    .email
                    .parse::<LettreAddress>()
                    .map_err(|_| MailServiceError::BuildMessageReplyToAddressFail)?,
            ));
        }

        builder = builder.to(Mailbox::new(
            self.to.name.to_owned(),
            self.to
                .email
                .parse::<LettreAddress>()
                .map_err(|_| MailServiceError::BuildMessageToAddressFail)?,
        ));

        builder = builder.subject(&self.subject);

        let message: Result<LettreMessage, MailServiceError> = match &self.html_body {
            Some(html_body) => builder
                .header(ContentType::TEXT_HTML)
                .body(html_body.to_string()),
            None => builder
                .header(ContentType::TEXT_PLAIN)
                .body(self.text_body.to_string()),
        }
        .map_err(|e| match e {
            LettreError::MissingFrom => MailServiceError::BuildMessageFromAddressFail,
            LettreError::MissingTo => MailServiceError::BuildMessageToAddressFail,
            LettreError::TooManyFrom => MailServiceError::BuildMessageFromAddressFail,
            _ => MailServiceError::BuildMessageFail,
        });

        message
    }
}
