use crate::{AuthToken, Session, SessionService, User, WebAuthService};
use actix_utils::future::{ready, Ready};
use actix_web::body::BoxBody;
use actix_web::web::Data;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use http::{HeaderValue, StatusCode};
use std::sync::Arc;
use std::{future::Future, pin::Pin, rc::Rc};

#[derive(Clone)]
pub struct WebMiddleware {
    auth: bool,
}

pub static REDIRECT_TO: &str = "/login";

impl<S> Transform<S, ServiceRequest> for WebMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = InnerWebMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InnerWebMiddleware {
            service: Rc::new(service),
            auth: self.auth,
        }))
    }
}

impl WebMiddleware {
    pub fn build(auth: bool) -> Self {
        Self { auth }
    }
}

pub struct InnerWebMiddleware<S> {
    service: Rc<S>,
    auth: bool,
}

fn unauthorized_redirect(auth_service: &WebAuthService) -> HttpResponse {
    HttpResponse::SeeOther()
        .cookie(auth_service.make_clear_cookie())
        .insert_header((
            http::header::LOCATION,
            HeaderValue::from_static(REDIRECT_TO),
        ))
        .finish()
}

impl<S> Service<ServiceRequest> for InnerWebMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let session_service: &Data<SessionService> =
            req.app_data::<Data<SessionService>>().unwrap();
        let session_service = Arc::clone(session_service);

        let session: Session = session_service.new_session_from_req(req.request());
        let session: Arc<Session> = Arc::new(session);
        let session_rc: Arc<Session> = Arc::clone(&session);
        req.extensions_mut().insert(Arc::clone(&session));

        if !self.auth {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let mut res: Self::Response = fut.await?;

                let cookie = session_service.make_cookie_throw_http(&session_rc)?;
                res.response_mut().add_cookie(&cookie).unwrap();

                Ok(res)
            });
        }

        let web_auth_service: &Data<WebAuthService> =
            req.app_data::<Data<WebAuthService>>().unwrap();
        let web_auth_service = Arc::clone(web_auth_service);

        let old_auth_token = web_auth_service.get_auth_token_from_request(req.request());

        if old_auth_token.is_none() {
            return Box::pin(async move {
                let res = unauthorized_redirect(web_auth_service.as_ref());
                Ok(req.into_response(res))
            });
        }
        let old_auth_token: AuthToken = old_auth_token.unwrap();

        let auth_data = web_auth_service.login_by_auth_token(&old_auth_token);

        if auth_data.is_err() {
            return Box::pin(async move {
                let res = unauthorized_redirect(web_auth_service.as_ref());
                Ok(req.into_response(res))
            });
        }

        let (user, new_auth_token) = auth_data.unwrap();
        let new_auth_token: Arc<AuthToken> = Arc::new(new_auth_token);
        let new_auth_token_rc: Arc<AuthToken> = Arc::clone(&new_auth_token);

        let user_rc: Arc<User> = Arc::new(user);
        req.extensions_mut().insert(Arc::clone(&user_rc));

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res: Self::Response = fut.await?;

            let cookie = session_service.make_cookie_throw_http(&session_rc)?;
            res.response_mut().add_cookie(&cookie).unwrap();

            match res.response().error() {
                Some(e) => {
                    // TODO: Render error page
                    if e.as_response_error()
                        .status_code()
                        .eq(&StatusCode::UNAUTHORIZED)
                    {
                        let res_mut = res.response_mut();

                        res_mut.head_mut().status = StatusCode::SEE_OTHER;

                        res_mut.headers_mut().insert(
                            http::header::LOCATION,
                            HeaderValue::from_static(REDIRECT_TO),
                        );

                        let _ = web_auth_service.expire_auth_token(new_auth_token_rc.as_ref());

                        let c = web_auth_service.make_clear_cookie();
                        let _ = res_mut.add_cookie(&c);

                        Ok(res)
                    } else {
                        Ok(res)
                    }
                }
                _ => {
                    let c = web_auth_service.make_cookie_throw_http(new_auth_token_rc.as_ref())?;
                    res.response_mut().add_cookie(&c).unwrap();

                    Ok(res)
                }
            }
        })
    }
}
