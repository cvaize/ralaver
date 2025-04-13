use crate::connections::Connections;
use crate::{
    AlertService, AppService, AuthService, Config, HashService, KeyValueService, LocaleService,
    LogService, MailService, RandomService, SessionService, TemplateService, TranslatorService,
};
use actix_web::web::Data;
use argon2::Argon2;

pub struct BaseServices {
    pub config: Data<Config>,
    pub log: Data<LogService>,
}

pub fn base(config: Config) -> BaseServices {
    BaseServices {
        config: Data::new(config),
        log: Data::new(LogService::new()),
    }
}

pub struct AdvancedServices<'a> {
    pub key_value: Data<KeyValueService>,
    pub translator: Data<TranslatorService>,
    pub template: Data<TemplateService>,
    pub session: Data<SessionService>,
    pub alert: Data<AlertService>,
    pub hash: Data<HashService<'a>>,
    pub auth: Data<AuthService<'a>>,
    pub locale: Data<LocaleService>,
    pub app: Data<AppService>,
    pub mail: Data<MailService>,
    pub rand: Data<RandomService>,
}

pub fn advanced<'a>(c: &Connections, s: &BaseServices) -> AdvancedServices<'a> {
    let key_value = Data::new(KeyValueService::new(c.redis.clone(), s.log.clone()));
    let translator = Data::new(
        TranslatorService::new_from_files(s.config.clone(), s.log.clone())
            .expect("Fail init TranslatorService::new_from_files"),
    );
    let template = Data::new(
        TemplateService::new_from_files(s.config.clone(), s.log.clone())
            .expect("Fail init TemplateService::new_from_files"),
    );
    let session = Data::new(SessionService::new(s.log.clone()));
    let alert = Data::new(AlertService::new(
        s.config.clone(),
        session.clone(),
        s.log.clone(),
    ));
    let hash = Data::new(HashService::new(Argon2::default(), s.log.clone()));
    let auth = Data::new(AuthService::new(
        s.config.clone(),
        c.mysql.clone(),
        hash.clone(),
        key_value.clone(),
        s.log.clone(),
        session.clone(),
    ));
    let locale = Data::new(LocaleService::new(s.config.clone(), session.clone()));
    let app = Data::new(AppService::new(
        s.config.clone(),
        locale.clone(),
        alert.clone(),
    ));
    let mail = Data::new(MailService::new(
        s.config.clone(),
        s.log.clone(),
        c.smtp.clone(),
    ));
    let rand = Data::new(RandomService::new());

    AdvancedServices {
        key_value,
        translator,
        template,
        session,
        alert,
        hash,
        auth,
        locale,
        app,
        mail,
        rand,
    }
}

#[allow(dead_code)]
pub struct Services<'a> {
    pub config: Data<Config>,
    pub log: Data<LogService>,
    pub key_value: Data<KeyValueService>,
    pub translator: Data<TranslatorService>,
    pub template: Data<TemplateService>,
    pub session: Data<SessionService>,
    pub alert: Data<AlertService>,
    pub hash: Data<HashService<'a>>,
    pub auth: Data<AuthService<'a>>,
    pub locale: Data<LocaleService>,
    pub app: Data<AppService>,
    pub mail: Data<MailService>,
    pub rand: Data<RandomService>,
}

pub fn join_to_all(base: BaseServices, advanced: AdvancedServices) -> Services {
    Services {
        config: base.config,
        log: base.log,
        key_value: advanced.key_value,
        translator: advanced.translator,
        template: advanced.template,
        session: advanced.session,
        alert: advanced.alert,
        hash: advanced.hash,
        auth: advanced.auth,
        locale: advanced.locale,
        app: advanced.app,
        mail: advanced.mail,
        rand: advanced.rand,
    }
}
