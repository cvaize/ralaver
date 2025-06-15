use crate::connections::Connections;
use crate::{
    AppService, AuthService, Config, CryptService, FileMysqlRepository, FileService, HashService,
    KeyValueService, LocaleService, MailService, RandomService, RateLimitService,
    RoleMysqlRepository, RoleService, TemplateService, TranslatorService, UserMysqlRepository,
    UserService, WebAuthService,
};
use actix_web::web::Data;

pub struct BaseServices {
    pub config: Data<Config>,
}

pub fn base(config: Config) -> BaseServices {
    BaseServices {
        config: Data::new(config),
    }
}

pub struct AdvancedServices {
    pub key_value_service: Data<KeyValueService>,
    pub translator_service: Data<TranslatorService>,
    pub template_service: Data<TemplateService>,
    pub crypt_service: Data<CryptService>,
    pub hash_service: Data<HashService>,
    pub web_auth_service: Data<WebAuthService>,
    pub auth_service: Data<AuthService>,
    pub locale_service: Data<LocaleService>,
    pub app_service: Data<AppService>,
    pub mail_service: Data<MailService>,
    pub rand_service: Data<RandomService>,
    pub rate_limit_service: Data<RateLimitService>,
    pub user_service: Data<UserService>,
    pub user_mysql_repository: Data<UserMysqlRepository>,
    pub role_service: Data<RoleService>,
    pub role_mysql_repository: Data<RoleMysqlRepository>,
    pub file_service: Data<FileService>,
    pub file_mysql_repository: Data<FileMysqlRepository>,
}

pub fn advanced(c: &Connections, s: &BaseServices) -> AdvancedServices {
    let key_value_service = Data::new(KeyValueService::new(c.redis.clone()));
    let translator_service = Data::new(
        TranslatorService::new_from_files(s.config.clone())
            .expect("Fail init TranslatorService::new_from_files"),
    );
    let template_service = Data::new(
        TemplateService::new_from_files(s.config.clone())
            .expect("Fail init TemplateService::new_from_files"),
    );
    let rand_service = Data::new(RandomService::new());

    let hash_service = Data::new(HashService::new(s.config.clone()));
    let user_mysql_repository = Data::new(UserMysqlRepository::new(c.mysql.clone()));
    let user_service = Data::new(UserService::new(
        hash_service.clone(),
        user_mysql_repository.clone(),
    ));

    let crypt_service = Data::new(CryptService::new(
        s.config.clone(),
        rand_service.clone(),
        hash_service.clone(),
    ));
    let auth_service = Data::new(AuthService::new(
        key_value_service.clone(),
        hash_service.clone(),
        user_service.clone(),
        user_mysql_repository.clone(),
    ));
    let locale_service = Data::new(LocaleService::new(s.config.clone()));
    let app_service = Data::new(AppService::new(s.config.clone(), locale_service.clone()));
    let mail_service = Data::new(MailService::new(s.config.clone(), c.smtp.clone()));
    let rate_limit_service = Data::new(RateLimitService::new(key_value_service.clone()));
    let web_auth_service = Data::new(WebAuthService::new(
        s.config.clone(),
        crypt_service.clone(),
        rand_service.clone(),
        key_value_service.clone(),
        hash_service.clone(),
        user_service.clone(),
    ));

    let role_mysql_repository = Data::new(RoleMysqlRepository::new(c.mysql.clone()));
    let role_service = Data::new(RoleService::new(role_mysql_repository.clone()));

    let file_mysql_repository = Data::new(FileMysqlRepository::new(c.mysql.clone()));
    let file_service = Data::new(FileService::new(file_mysql_repository.clone()));

    AdvancedServices {
        key_value_service,
        translator_service,
        template_service,
        hash_service,
        crypt_service,
        web_auth_service,
        auth_service,
        locale_service,
        app_service,
        mail_service,
        rand_service,
        rate_limit_service,
        user_service,
        user_mysql_repository,
        role_service,
        role_mysql_repository,
        file_service,
        file_mysql_repository,
    }
}

pub struct Services {
    pub config: Data<Config>,
    pub key_value_service: Data<KeyValueService>,
    pub translator_service: Data<TranslatorService>,
    pub template_service: Data<TemplateService>,
    pub hash_service: Data<HashService>,
    pub web_auth_service: Data<WebAuthService>,
    pub auth_service: Data<AuthService>,
    pub crypt_service: Data<CryptService>,
    pub locale_service: Data<LocaleService>,
    pub app_service: Data<AppService>,
    pub mail_service: Data<MailService>,
    pub rand_service: Data<RandomService>,
    pub rate_limit_service: Data<RateLimitService>,
    pub user_service: Data<UserService>,
    pub user_mysql_repository: Data<UserMysqlRepository>,
    pub role_service: Data<RoleService>,
    pub role_mysql_repository: Data<RoleMysqlRepository>,
    pub file_service: Data<FileService>,
    pub file_mysql_repository: Data<FileMysqlRepository>,
}

pub fn join_to_all(base: BaseServices, advanced: AdvancedServices) -> Services {
    Services {
        config: base.config,
        key_value_service: advanced.key_value_service,
        translator_service: advanced.translator_service,
        template_service: advanced.template_service,
        hash_service: advanced.hash_service,
        web_auth_service: advanced.web_auth_service,
        auth_service: advanced.auth_service,
        crypt_service: advanced.crypt_service,
        locale_service: advanced.locale_service,
        app_service: advanced.app_service,
        mail_service: advanced.mail_service,
        rand_service: advanced.rand_service,
        rate_limit_service: advanced.rate_limit_service,
        user_service: advanced.user_service,
        user_mysql_repository: advanced.user_mysql_repository,
        role_service: advanced.role_service,
        role_mysql_repository: advanced.role_mysql_repository,
        file_service: advanced.file_service,
        file_mysql_repository: advanced.file_mysql_repository,
    }
}
