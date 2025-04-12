use crate::{Config, LogService};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use strum_macros::{Display, EnumString};
pub use lettre::{
    Address as LettreAddress, Message as LettreMessage, SmtpTransport as LettreSmtpTransport,
    Transport as LettreTransport,
};
pub use lettre::message::header::ContentType as LettreContentType;
pub use lettre::message::Mailbox as LettreMailbox;
pub use lettre::error::Error as LettreError;

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum SmtpConnectionError {
    ParsePortFail,
    MakeTlsParametersFail,
    EncryptionNotSupported,
}

pub fn get_smtp_transport(
    config: &Config,
    log_service: &LogService,
) -> Result<LettreSmtpTransport, SmtpConnectionError> {
    log_service.info("Make smtp transport.");

    let smtp = &config.mail.smtp;
    let host = smtp.host.to_owned();
    let port: u16 = smtp.port.to_owned().parse().map_err(|e| {
        log_service.error(format!("SmtpConnectionError::ParsePortFail - {:}", &e).as_str());
        SmtpConnectionError::ParsePortFail
    })?;
    let username = smtp.username.to_owned();
    let password = smtp.password.to_owned();

    let creds = Credentials::new(username, password);

    let mut mailer = LettreSmtpTransport::builder_dangerous(host.to_owned()).port(port);

    if smtp.encryption == "" {
    } else if smtp.encryption == "tls" {
        let tls_parameters = TlsParameters::new(host.into()).map_err(|e| {
            log_service
                .error(format!("SmtpConnectionError::MakeTlsParametersFail - {:}", &e).as_str());
            SmtpConnectionError::MakeTlsParametersFail
        })?;

        mailer = mailer.tls(Tls::Wrapper(tls_parameters));
    } else {
        log_service.error(
            format!(
                "SmtpConnectionError::EncryptionNotSupported - {}",
                &smtp.encryption
            )
            .as_str(),
        );
        return Err(SmtpConnectionError::EncryptionNotSupported);
    }

    let mailer = mailer.credentials(creds).build();

    Ok(mailer)
}
