use crate::{Alert, AlertService, Config, LocaleService};
use crate::{Locale, User};
use actix_session::Session;
use actix_web::web::Data;
use actix_web::HttpRequest;
use url::Url;

pub struct AppService {
    config: Data<Config>,
    url: Url,
    locale_service: Data<LocaleService>,
    alert_service: Data<AlertService>,
}

impl AppService {
    pub fn new(config: Data<Config>, locale_service: Data<LocaleService>, alert_service: Data<AlertService>) -> Self {
        let url: Url = Url::parse(&config.get_ref().app.url).unwrap();
        Self {
            config,
            url,
            locale_service,
            alert_service,
        }
    }

    pub fn locale(
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
    pub fn dark_mode(&self, req: &HttpRequest) -> Option<String> {
        req.cookie(&self.config.get_ref().app.dark_mode_cookie_key)
            .map(|c| c.value().to_owned())
    }

    pub fn alerts(&self, session: &Session) -> Vec<Alert> {
        self.alert_service.get_and_remove_from_session(session).unwrap_or(Vec::new())
    }

    pub fn url(&self) -> &Url {
        &self.url
    }
}
