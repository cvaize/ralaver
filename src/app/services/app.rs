use crate::{Config, LocaleService};
use crate::{Locale, User};
use actix_session::Session;
use actix_web::web::Data;
use actix_web::HttpRequest;

#[derive(Debug)]
pub struct AppService {
    config: Data<Config>,
    locale_service: Data<LocaleService>,
}

impl AppService {
    pub fn new(config: Data<Config>, locale_service: Data<LocaleService>) -> Self {
        Self {
            config,
            locale_service,
        }
    }

    pub fn get_locale(
        &self,
        req: Option<&HttpRequest>,
        session: Option<&Session>,
        user: Option<&User>,
    ) -> (String, &Locale, &Vec<Locale>) {
        let locale_service = self.locale_service.get_ref();
        let lang: String = locale_service.get_locale_code(req, session, user);
        let locale: &Locale = locale_service.get_locale_or_default_ref(&lang);
        let locales_without_current: &Vec<Locale> =
            locale_service.get_locales_or_default_without_current_ref(&lang);

        (lang, locale, locales_without_current)
    }

    // Return "dark" or "light" or None
    pub fn get_dark_mode(&self, req: &HttpRequest) -> Option<String> {
        req.cookie(&self.config.app.dark_mode_cookie_key)
            .map(|c| c.value().to_owned())
    }
}
