use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use actix_web::body::EitherBody;
use futures::future::LocalBoxFuture;
use std::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use sqlx::PgPool;
use std::{
    rc::Rc,
    task::{Context, Poll}
};
use std::sync::Arc;
use crate::models::{Users, Claims};


#[derive(Clone)]
pub struct AuthMiddleware {
    pool: Arc<PgPool>,
    jwt_secret: String,
}

impl AuthMiddleware {
    pub fn new(pool: PgPool, jwt_secret: String) -> Self {
        Self {
            pool: Arc::new(pool),
            jwt_secret,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareMiddleware {
            service,
            pool: self.pool.clone(),
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}


pub struct AuthMiddlewareMiddleware<S> {
    service: S,
    pool: Arc<PgPool>,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let pool = self.pool.clone();
        let jwt_secret = self.jwt_secret.clone();
        let svc = self.service.clone();

        Box::pin(async move {
            // Extract Authorization: Bearer token
            let token = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
                .ok_or_else(|| actix_web::error::ErrorUnauthorized("No token provided"))?;

            // Decode JWT
            let decoded = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(jwt_secret.as_bytes()),
                &Validation::new(Algorithm::HS256),
            )
            .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid or expired token"))?;

            // Check revocation
            let revoked = sqlx::query_scalar::<_, i64>(
                "SELECT 1 FROM revoked_tokens WHERE token = $1"
            )
            .bind(&token)
            .fetch_optional(&*pool)
            .await
            .map_err(|_| actix_web::error::ErrorInternalServerError("DB error"))?;

            if revoked.is_some() {
                return Err(actix_web::error::ErrorUnauthorized("Token revoked"));
            }

            // Fetch user
            let user = sqlx::query_as::<_, Users>(
                "SELECT * FROM users WHERE id = $1"
            )
            .bind(decoded.claims.sub)
            .fetch_one(&*pool)
            .await
            .map_err(|_| actix_web::error::ErrorUnauthorized("User not found"))?;

            // Attach user
            req.extensions_mut().insert(user);

            let res = svc.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}
