use crate::User;
use crate::{Config, SessionService};
use actix_session::Session;
use actix_web::web::Data;
use actix_web::HttpRequest;
use garde::rules::length::simple::Simple;

#[derive(Debug)]
pub struct AppService {
    config: Data<Config>,
    session_service: Data<SessionService>,
}

impl AppService {
    pub fn new(config: Data<Config>, session_service: Data<SessionService>) -> Self {
        Self {
            config,
            session_service,
        }
    }

    pub fn get_locale(
        &self,
        req: Option<&HttpRequest>,
        session: Option<&Session>,
        user: Option<&User>,
    ) -> String {
        if let Some(req) = req {
            if let Some(locale) = req.cookie(&self.config.app.locale_cookie_key) {
                let locale = locale.value().to_string();
                if locale.validate_length(1, 6).is_ok() {
                    return locale;
                }
            }
        }
        if let Some(session) = session {
            let locale = self
                .session_service
                .get_ref()
                .get_string(session, &self.config.app.locale_session_key);
            if let Ok(Some(locale)) = locale {
                if locale.validate_length(1, 6).is_ok() {
                    return locale;
                }
            }
        }
        if let Some(user) = user {
            if let Some(locale) = &user.locale {
                if locale.validate_length(1, 6).is_ok() {
                    return locale.to_string();
                }
            }
        }

        self.config.app.locale.to_string()
    }

    // Return "dark" or "light" or None
    pub fn get_dark_mode(&self, req: &HttpRequest) -> Option<String> {
        req.cookie(&self.config.app.dark_mode_cookie_key).map(|c| c.value().to_owned())
    }
}
