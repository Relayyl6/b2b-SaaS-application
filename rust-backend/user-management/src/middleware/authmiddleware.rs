use actix_web::body::EitherBody;
use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use sqlx::PgPool;
use std::{
    future::{Ready, ready},
    rc::Rc,
};

use crate::models::{Claims, Users};

#[derive(Clone)]
pub struct AuthMiddleware {
    pool: PgPool,
    jwt_secret: String,
    redis_client: Option<redis::Client>,
}

impl AuthMiddleware {
    pub fn new(pool: PgPool, jwt_secret: String, redis_client: Option<redis::Client>) -> Self {
        Self { pool, jwt_secret, redis_client }
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
            service: Rc::new(service),
            pool: self.pool.clone(),
            jwt_secret: self.jwt_secret.clone(),
            redis_client: self.redis_client.clone(),
        }))
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
    pool: PgPool,
    jwt_secret: String,
    redis_client: Option<redis::Client>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
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
        let svc = self.service.clone();
        let pool = self.pool.clone();
        let jwt_secret = self.jwt_secret.clone();
        let redis_client = self.redis_client.clone();

        Box::pin(async move {
            let token = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .map(|s| s.to_string())
                .ok_or_else(|| actix_web::error::ErrorUnauthorized("No token provided"))?;

            let decoded = decode::<Claims>(
                &token,
                &DecodingKey::from_secret(jwt_secret.as_bytes()),
                &Validation::new(Algorithm::HS256),
            )
            .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid or expired token"))?;

            let mut is_revoked = false;
            if let Some(client) = &redis_client {
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    let redis_key = format!("revoked_token:{}", token);
                    if let Ok(true) = redis::cmd("EXISTS").arg(&redis_key).query_async::<_, bool>(&mut conn).await {
                        is_revoked = true;
                    }
                }
            } else {
                let revoked =
                    sqlx::query_scalar::<_, i64>("SELECT 1 FROM revoked_tokens WHERE token = $1 LIMIT 1")
                        .bind(&token)
                        .fetch_optional(&pool)
                        .await
                        .unwrap_or(None);
                if revoked.is_some() {
                    is_revoked = true;
                }
            }

            if is_revoked {
                return Err(actix_web::error::ErrorUnauthorized("Token revoked"));
            }

            let user = sqlx::query_as::<_, Users>("SELECT * FROM users WHERE id = $1")
                .bind(decoded.claims.sub)
                .fetch_one(&pool)
                .await
                .map_err(|_| actix_web::error::ErrorUnauthorized("User not found"))?;

            req.extensions_mut().insert(user);

            let res = svc.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}
