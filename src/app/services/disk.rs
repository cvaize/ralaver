use crate::{
    TranslatableError, TranslatorService,
};
use strum_macros::{Display, EnumString};

pub struct DiskService {
    // https://laravel.com/docs/master/filesystem
    // TODO: DiskService as Laravel/Storage
}

impl DiskService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn put(){}
    pub fn mv(){}
    pub fn cp(){}
}

#[derive(Debug, Clone, Copy, Display, EnumString, PartialEq, Eq)]
pub enum DiskServiceError {
    NotFound,
    Fail,
}

impl TranslatableError for DiskServiceError {
    fn translate(&self, lang: &str, translator_service: &TranslatorService) -> String {
        match self {
            Self::NotFound => translator_service.translate(lang, "error.DiskServiceError.NotFound"),
            _ => translator_service.translate(lang, "error.DiskServiceError.Fail"),
        }
    }
}
