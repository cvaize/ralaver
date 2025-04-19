use crate::app::connections::smtp::{
    LettreAddress, LettreContentType, LettreError, LettreMailbox, LettreMessage,
    LettreSmtpTransport, LettreTransport,
};
use crate::config::MailSmtpConfig;
use crate::{Config};
use actix_web::web::Data;
use strum_macros::{Display, EnumString};


pub struct MailService {
    config: Data<Config>,
    mailer: Data<LettreSmtpTransport>,
}

impl MailService {
    pub fn new(
        config: Data<Config>,
        mailer: Data<LettreSmtpTransport>,
    ) -> Self {
        Self {
            config,
            mailer,
        }
    }

    pub fn send_email(&self, data: &EmailMessage) -> Result<(), MailServiceError> {
        let mailer = &self.mailer;
        let message: LettreMessage = data.build(&self.config.get_ref().mail.smtp).map_err(|e| {
            log::error!("{}",format!("MailService::send_email - {:}", &e).as_str());
            e
        })?;

        mailer.send(&message).map_err(|e| {
            log::error!("{}",format!("MailService::send_email - {:}", &e).as_str());
            MailServiceError::SendFail
        })?;

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

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum MailServiceError {
    SendFail,
    BuildMessageFromAddressFail,
    BuildMessageReplyToAddressFail,
    BuildMessageToAddressFail,
    BuildMessageBodyFail,
    BuildMessageFail,
}

impl EmailMessage {
    fn build(&self, smtp: &MailSmtpConfig) -> Result<LettreMessage, MailServiceError> {
        let mut builder = LettreMessage::builder();

        if let Some(from) = &self.from {
            builder = builder.from(LettreMailbox::new(
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
                builder = builder.from(LettreMailbox::new(
                    from_name,
                    from_address
                        .parse::<LettreAddress>()
                        .map_err(|_| MailServiceError::BuildMessageFromAddressFail)?,
                ));
            }
        }

        if let Some(reply_to) = &self.reply_to {
            builder = builder.reply_to(LettreMailbox::new(
                reply_to.name.to_owned(),
                reply_to
                    .email
                    .parse::<LettreAddress>()
                    .map_err(|_| MailServiceError::BuildMessageReplyToAddressFail)?,
            ));
        }

        builder = builder.to(LettreMailbox::new(
            self.to.name.to_owned(),
            self.to
                .email
                .parse::<LettreAddress>()
                .map_err(|_| MailServiceError::BuildMessageToAddressFail)?,
        ));

        builder = builder.subject(&self.subject);

        let message: Result<LettreMessage, MailServiceError> = match &self.html_body {
            Some(html_body) => builder
                .header(LettreContentType::TEXT_HTML)
                .body(html_body.to_string()),
            None => builder
                .header(LettreContentType::TEXT_PLAIN)
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
