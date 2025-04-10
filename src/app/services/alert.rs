use crate::{Alert, Config, SessionService};
use actix_session::Session;
use actix_web::web::Data;

pub struct AlertService {
    config: Config,
    session_service: Data<SessionService>,
}

impl AlertService {
    pub fn new(config: Config, session_service: Data<SessionService>) -> Self {
        Self {
            config,
            session_service,
        }
    }

    pub fn insert_into_session(
        &self,
        session: &Session,
        alerts: &Vec<Alert>,
    ) -> Result<(), SaveAlertsError> {
        self.session_service
            .insert(session, &self.config.alerts.session_key, alerts)
            .map_err(|_| SaveAlertsError)?;
        Ok(())
    }

    pub fn get_from_session(&self, session: &Session) -> Result<Vec<Alert>, GetAlertsError> {
        let alerts: Vec<Alert> = self
            .session_service
            .get(session, &self.config.alerts.session_key)
            .map_err(|_| GetAlertsError)?
            .unwrap_or(Vec::new());
        Ok(alerts)
    }

    pub fn remove_from_session(&self, session: &Session) {
        self.session_service.remove(session, &self.config.alerts.session_key);
    }

    pub fn get_and_remove_from_session(
        &self,
        session: &Session,
    ) -> Result<Vec<Alert>, GetAlertsError> {
        let alerts: Vec<Alert> = self.get_from_session(session)?;
        self.remove_from_session(session);
        Ok(alerts)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SaveAlertsError;
#[derive(Debug, Clone, Copy)]
pub struct GetAlertsError;
