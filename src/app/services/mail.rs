use crate::config::MailSmtpConfig;
use crate::{Config, LogService};
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
use strum_macros::{Display, EnumString};

pub struct MailService {
    config: Config,
    mailer: LettreSmtpTransport,
    log_service: Data<LogService>,
}

impl MailService {
    pub fn new(
        config: Config,
        log_service: Data<LogService>,
        mailer: Option<LettreSmtpTransport>,
    ) -> Self {
        let log_service_ref = log_service.get_ref();
        let mailer = match mailer {
            Some(mailer) => mailer,
            _ => Self::connect(log_service_ref, &config)
                .map_err(|e| {
                    log_service.error(format!("MailService::new - {:}", &e).as_str());
                    e
                })
                .unwrap(),
        };
        Self {
            config,
            mailer,
            log_service,
        }
    }

    pub fn send_email(&self, data: &EmailMessage) -> Result<(), MailServiceError> {
        let mailer = &self.mailer;
        let message: LettreMessage = data.build(&self.config.mail.smtp).map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("MailService::send_email - {:}", &e).as_str());
            e
        })?;

        mailer.send(&message).map_err(|e| {
            self.log_service
                .get_ref()
                .error(format!("MailService::send_email - {:}", &e).as_str());
            MailServiceError::SendFail
        })?;

        Ok(())
    }

    pub fn connect(
        log_service: &LogService,
        config: &Config,
    ) -> Result<LettreSmtpTransport, MailServiceError> {
        let host = config.mail.smtp.host.to_owned();
        let port: u16 = config.mail.smtp.port.to_owned().parse().map_err(|e| {
            log_service.error(format!("MailService::connect - {:}", &e).as_str());
            MailServiceError::ConnectSmtpFail
        })?;
        let username = config.mail.smtp.username.to_owned();
        let password = config.mail.smtp.password.to_owned();

        let creds = Credentials::new(username, password);

        let mut mailer = LettreSmtpTransport::builder_dangerous(host.to_owned()).port(port);

        if config.mail.smtp.encryption == "" {
        } else if config.mail.smtp.encryption == "tls" {
            let tls_parameters = TlsParameters::new(host.into()).map_err(|e| {
                log_service.error(format!("MailService::connect - {:}", &e).as_str());
                MailServiceError::ConnectSmtpFail
            })?;

            mailer = mailer.tls(Tls::Wrapper(tls_parameters));
        } else {
            return Err(MailServiceError::EncryptionNotSupported);
        }

        let mailer = mailer.credentials(creds).build();

        Ok(mailer)
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

#[derive(Debug, Clone, Copy, Display, EnumString)]
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
