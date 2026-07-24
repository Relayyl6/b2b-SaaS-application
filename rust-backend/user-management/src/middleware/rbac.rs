use actix_web::body::EitherBody;
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{Ready, ready},
    rc::Rc,
};

use crate::models::{UserRole, Users};

#[derive(Clone)]
pub struct RequireRole {
    allowed_roles: Vec<UserRole>,
}

impl RequireRole {
    pub fn new(allowed_roles: Vec<UserRole>) -> Self {
        Self { allowed_roles }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequireRole
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RequireRoleMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequireRoleMiddleware {
            service: Rc::new(service),
            allowed_roles: self.allowed_roles.clone(),
        }))
    }
}

pub struct RequireRoleMiddleware<S> {
    service: Rc<S>,
    allowed_roles: Vec<UserRole>,
}

impl<S, B> Service<ServiceRequest> for RequireRoleMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let extensions = req.extensions();
        let user = extensions.get::<Users>();

        if let Some(user) = user {
            if !self.allowed_roles.contains(&user.role) {
                return Box::pin(ready(Err(actix_web::error::ErrorForbidden(
                    "User does not have required role",
                ))));
            }
        } else {
            return Box::pin(ready(Err(actix_web::error::ErrorUnauthorized(
                "User not authenticated",
            ))));
        }

        drop(extensions);

        let svc = self.service.clone();
        Box::pin(async move {
            let res = svc.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}
