use crate::{Alert, Config, LogService, SessionService};
use actix_session::Session;
use actix_web::web::Data;
use strum_macros::{Display, EnumString};

pub struct AlertService {
    config: Data<Config>,
    session_service: Data<SessionService>,
    log_service: Data<LogService>,
}

impl AlertService {
    pub fn new(
        config: Data<Config>,
        session_service: Data<SessionService>,
        log_service: Data<LogService>,
    ) -> Self {
        Self {
            config,
            session_service,
            log_service,
        }
    }

    pub fn insert_into_session(
        &self,
        session: &Session,
        alerts: &Vec<Alert>,
    ) -> Result<(), AlertServiceError> {
        self.session_service
            .insert(session, &self.config.get_ref().alerts.session_key, alerts)
            .map_err(|e| {
                self.log_service
                    .get_ref()
                    .error(format!("AlertService::insert_into_session - {:}", &e).as_str());
                return AlertServiceError::InsertFail;
            })?;
        Ok(())
    }

    pub fn get_from_session(&self, session: &Session) -> Result<Vec<Alert>, AlertServiceError> {
        let alerts: Vec<Alert> = self
            .session_service
            .get(session, &self.config.get_ref().alerts.session_key)
            .map_err(|e| {
                self.log_service
                    .get_ref()
                    .error(format!("AlertService::get_from_session - {:}", &e).as_str());
                return AlertServiceError::GetFail;
            })?
            .unwrap_or(Vec::new());
        Ok(alerts)
    }

    pub fn remove_from_session(&self, session: &Session) {
        self.session_service
            .remove(session, &self.config.get_ref().alerts.session_key);
    }

    pub fn get_and_remove_from_session(
        &self,
        session: &Session,
    ) -> Result<Vec<Alert>, AlertServiceError> {
        let alerts: Vec<Alert> = self.get_from_session(session)?;
        self.remove_from_session(session);
        Ok(alerts)
    }
}

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum AlertServiceError {
    InsertFail,
    GetFail,
}
