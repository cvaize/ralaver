use crate::connections::Connections;
use crate::{AppService, AuthService, Config, CryptService, DiskExternalRepository, DiskLocalRepository, FileMysqlRepository, FileService, HashService, KeyValueService, LocaleService, MailService, RandomService, RateLimitService, RoleMysqlRepository, RoleService, TemplateService, TranslatorService, UserFileMysqlRepository, UserFileService, UserMysqlRepository, UserService, WebAuthService};
use actix_web::web::Data;
use std::path::MAIN_SEPARATOR_STR;

pub struct Services {
    pub config: Data<Config>,
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
    pub disk_local_repository: Data<DiskLocalRepository>,
    pub disk_external_repository: Data<DiskExternalRepository>,
    pub file_service: Data<FileService>,
    pub file_mysql_repository: Data<FileMysqlRepository>,
    pub user_file_service: Data<UserFileService>,
    pub user_file_mysql_repository: Data<UserFileMysqlRepository>,
}

pub fn build(c: &Connections, config: Data<Config>) -> Services {
    let key_value_service = Data::new(KeyValueService::new(c.redis.clone()));
    let translator_service = Data::new(
        TranslatorService::new_from_files(config.clone())
            .expect("Fail init TranslatorService::new_from_files"),
    );
    let template_service = Data::new(
        TemplateService::new_from_files(config.clone())
            .expect("Fail init TemplateService::new_from_files"),
    );
    let rand_service = Data::new(RandomService::new());

    let hash_service = Data::new(HashService::new(config.clone()));
    let user_mysql_repository = Data::new(UserMysqlRepository::new(c.mysql.clone()));
    let user_service = Data::new(UserService::new(
        hash_service.clone(),
        user_mysql_repository.clone(),
    ));

    let crypt_service = Data::new(CryptService::new(
        config.clone(),
        rand_service.clone(),
        hash_service.clone(),
    ));
    let auth_service = Data::new(AuthService::new(
        key_value_service.clone(),
        hash_service.clone(),
        user_service.clone(),
    ));
    let locale_service = Data::new(LocaleService::new(config.clone()));
    let app_service = Data::new(AppService::new(config.clone(), locale_service.clone()));
    let mail_service = Data::new(MailService::new(config.clone(), c.smtp.clone()));
    let rate_limit_service = Data::new(RateLimitService::new(key_value_service.clone()));
    let web_auth_service = Data::new(WebAuthService::new(
        config.clone(),
        crypt_service.clone(),
        rand_service.clone(),
        key_value_service.clone(),
        hash_service.clone(),
        user_service.clone(),
    ));

    let role_mysql_repository = Data::new(RoleMysqlRepository::new(c.mysql.clone()));
    let role_service = Data::new(RoleService::new(role_mysql_repository.clone()));

    let disk_local_repository = Data::new(DiskLocalRepository::new(
        &config.get_ref().filesystem.disks.local.root,
        &config.get_ref().filesystem.disks.local.public_root,
        MAIN_SEPARATOR_STR,
    ));
    let disk_external_repository = Data::new(DiskExternalRepository::new());

    let file_mysql_repository = Data::new(FileMysqlRepository::new(c.mysql.clone()));
    let user_file_mysql_repository = Data::new(UserFileMysqlRepository::new(c.mysql.clone()));
    let user_file_service = Data::new(UserFileService::new(
        config.clone(),
        user_file_mysql_repository.clone(),
        disk_local_repository.clone(),
    ));
    let file_service = Data::new(FileService::new(
        config.clone(),
        file_mysql_repository.clone(),
        user_file_service.clone(),
        disk_local_repository.clone(),
        disk_external_repository.clone(),
        rand_service.clone(),
    ));

    Services {
        config,
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
        disk_local_repository,
        disk_external_repository,
        file_service,
        file_mysql_repository,
        user_file_service,
        user_file_mysql_repository,
    }
}
