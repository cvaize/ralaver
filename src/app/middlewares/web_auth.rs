use crate::{Session, User, WebAuthService};
use actix_utils::future::{ready, Ready};
use actix_web::body::BoxBody;
use actix_web::web::Data;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        header::{HeaderValue, LOCATION},
        StatusCode,
    },
    Error, HttpMessage, HttpResponse,
};
use std::sync::Arc;
use std::{future::Future, pin::Pin, rc::Rc};

#[derive(Clone)]
pub struct WebAuthMiddleware;

pub static REDIRECT_TO: &str = "/login";

impl<S> Transform<S, ServiceRequest> for WebAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = InnerWebAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(InnerWebAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct InnerWebAuthMiddleware<S> {
    service: Rc<S>,
}

fn unauthorized_redirect(auth_service: &WebAuthService) -> HttpResponse {
    HttpResponse::SeeOther()
        .cookie(auth_service.make_clear_cookie())
        .insert_header((LOCATION, HeaderValue::from_static(REDIRECT_TO)))
        .finish()
}

impl<S> Service<ServiceRequest> for InnerWebAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let web_auth_service: &Data<WebAuthService> =
            req.app_data::<Data<WebAuthService>>().unwrap();
        let web_auth_service = Arc::clone(web_auth_service);

        let old_session = web_auth_service.get_session_from_request(req.request());

        if old_session.is_none() {
            return Box::pin(async move {
                let res = unauthorized_redirect(web_auth_service.as_ref());
                Ok(req.into_response(res))
            });
        }
        let old_session: Session = old_session.unwrap();

        let auth_data = web_auth_service.login_by_session(&old_session);

        if auth_data.is_err() {
            return Box::pin(async move {
                let res = unauthorized_redirect(web_auth_service.as_ref());
                Ok(req.into_response(res))
            });
        }

        let (user, new_session) = auth_data.unwrap();
        let new_session: Arc<Session> = Arc::new(new_session);
        let new_session_rc: Arc<Session> = Arc::clone(&new_session);
        req.extensions_mut().insert(Arc::clone(&new_session));

        let user_rc: Arc<User> = Arc::new(user);
        req.extensions_mut().insert(Arc::clone(&user_rc));

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res: Self::Response = fut.await?;

            match res.response().error() {
                Some(e) => {
                    // TODO: Render error page
                    if e.as_response_error()
                        .status_code()
                        .eq(&StatusCode::UNAUTHORIZED)
                    {
                        let res_mut = res.response_mut();

                        res_mut.head_mut().status = StatusCode::SEE_OTHER;

                        res_mut
                            .headers_mut()
                            .insert(LOCATION, HeaderValue::from_static(REDIRECT_TO));

                        let _ = web_auth_service.expire_session(new_session_rc.as_ref());

                        let c = web_auth_service.make_clear_cookie();
                        let _ = res_mut.add_cookie(&c);

                        Ok(res)
                    } else {
                        Ok(res)
                    }
                }
                _ => {
                    let c = web_auth_service.make_cookie_throw_http(new_session_rc.as_ref())?;
                    res.response_mut().add_cookie(&c).unwrap();

                    Ok(res)
                }
            }
        })
    }
}
