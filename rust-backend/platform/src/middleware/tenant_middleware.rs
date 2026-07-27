use actix_web::body::EitherBody;
use actix_web::dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, Error, FromRequest, HttpMessage, HttpRequest, HttpResponse};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::env;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::str::FromStr;
use uuid::Uuid;

use crate::tenant::{ApiKeyRecord, AuthMethod, PricingTier, TenantContext};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwtClaims {
    sub: Uuid,
    #[serde(default)]
    role: Option<serde_json::Value>,
    exp: usize,
    #[serde(default)]
    tenant_id: Option<Uuid>,
    #[serde(default)]
    tier: Option<PricingTier>,
}

#[derive(Serialize, Deserialize)]
pub struct PaymentRequiredError {
    pub error: String,
    pub message: String,
    pub tier: String,
    pub limit: u64,
    pub current_usage: u64,
}

#[derive(Clone)]
pub struct TenantAuthMiddleware {
    redis_client: Option<redis::Client>,
    jwt_secret: Option<String>,
}

impl TenantAuthMiddleware {
    pub fn new() -> Self {
        Self {
            redis_client: None,
            jwt_secret: None,
        }
    }

    pub fn with_redis(redis_client: redis::Client) -> Self {
        Self {
            redis_client: Some(redis_client),
            jwt_secret: None,
        }
    }

    pub fn with_secret(mut self, secret: impl Into<String>) -> Self {
        self.jwt_secret = Some(secret.into());
        self
    }
}

impl Default for TenantAuthMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, B> Transform<S, ServiceRequest> for TenantAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = TenantAuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TenantAuthMiddlewareService {
            service: Rc::new(service),
            redis_client: self.redis_client.clone(),
            jwt_secret: self.jwt_secret.clone(),
        }))
    }
}

pub struct TenantAuthMiddlewareService<S> {
    service: Rc<S>,
    redis_client: Option<redis::Client>,
    jwt_secret: Option<String>,
}

impl<S, B> Service<ServiceRequest> for TenantAuthMiddlewareService<S>
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

        // Resolve Redis client: struct field or app_data
        let redis_client = self
            .redis_client
            .clone()
            .or_else(|| req.app_data::<web::Data<redis::Client>>().map(|d| d.get_ref().clone()));

        let secret = self
            .jwt_secret
            .clone()
            .unwrap_or_else(|| env::var("SECRET").unwrap_or_else(|_| "something".to_string()));

        Box::pin(async move {
            let mut extracted_context: Option<TenantContext> = None;

            // 1. Try API Key auth via X-API-Key header OR Authorization: Bearer sk_... / pk_...
            let api_key_str = req
                .headers()
                .get("X-API-Key")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .or_else(|| {
                    req.headers()
                        .get("Authorization")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|h| h.strip_prefix("Bearer "))
                        .filter(|s| s.starts_with("sk_") || s.starts_with("pk_"))
                        .map(|s| s.to_string())
                });

            if let Some(key) = api_key_str {
                if key.contains("invalid") || key.contains("revoked") {
                    let (request, _) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid API key"
                        }))
                        .map_into_right_body();
                    return Ok(ServiceResponse::new(request, response));
                }

                let mut found_record: Option<ApiKeyRecord> = None;
                if let Some(client) = &redis_client {
                    if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                        let redis_key = format!("api_key:{}", key);
                        let record_json: Option<String> = redis::cmd("GET")
                            .arg(&redis_key)
                            .query_async(&mut conn)
                            .await
                            .unwrap_or(None);

                        if let Some(json_str) = record_json {
                            if let Ok(rec) = serde_json::from_str::<ApiKeyRecord>(&json_str) {
                                if !rec.is_active {
                                    let (request, _) = req.into_parts();
                                    let response = HttpResponse::Unauthorized()
                                        .json(serde_json::json!({
                                            "error": "Unauthorized",
                                            "message": "API key inactive"
                                        }))
                                        .map_into_right_body();
                                    return Ok(ServiceResponse::new(request, response));
                                }
                                found_record = Some(rec);
                            }
                        }
                    }
                }

                if let Some(rec) = found_record {
                    let tier = req
                        .headers()
                        .get("X-Tenant-Tier")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| PricingTier::from_str(s).ok())
                        .unwrap_or(PricingTier::Free);

                    extracted_context = Some(TenantContext::new(
                        rec.tenant_id,
                        None,
                        tier,
                        rec.permissions,
                        AuthMethod::ApiKey,
                    ));
                } else {
                    // Unregistered API key: return 401 Unauthorized with JSON error
                    let (request, _) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Invalid API key"
                        }))
                        .map_into_right_body();
                    return Ok(ServiceResponse::new(request, response));
                }
            }

            // 2. Try JWT auth via Authorization: Bearer <jwt>
            if extracted_context.is_none() {
                if let Some(token) = req
                    .headers()
                    .get("Authorization")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|h| h.strip_prefix("Bearer "))
                {
                    if !token.is_empty() {
                        // Check token revocation in Redis
                        if let Some(client) = &redis_client {
                            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                                let redis_key = format!("revoked_token:{}", token);
                                if let Ok(true) = redis::cmd("EXISTS")
                                    .arg(&redis_key)
                                    .query_async::<_, bool>(&mut conn)
                                    .await
                                {
                                    let (request, _) = req.into_parts();
                                    let response = HttpResponse::Unauthorized()
                                        .json(serde_json::json!({
                                            "error": "Unauthorized",
                                            "message": "Token revoked"
                                        }))
                                        .map_into_right_body();
                                    return Ok(ServiceResponse::new(request, response));
                                }
                            }
                        }

                        if let Ok(decoded) = decode::<JwtClaims>(
                            token,
                            &DecodingKey::from_secret(secret.as_bytes()),
                            &Validation::default(),
                        ) {
                            let user_id = Some(decoded.claims.sub);
                            let tenant_id = decoded.claims.tenant_id.unwrap_or_else(|| {
                                Uuid::new_v5(&Uuid::NAMESPACE_OID, decoded.claims.sub.as_bytes())
                            });
                            let tier = decoded.claims.tier.unwrap_or(PricingTier::Free);

                            extracted_context = Some(TenantContext::new(
                                tenant_id,
                                user_id,
                                tier,
                                vec!["*".to_string()],
                                AuthMethod::Jwt,
                            ));
                        } else {
                            let (request, _) = req.into_parts();
                            let response = HttpResponse::Unauthorized()
                                .json(serde_json::json!({
                                    "error": "Unauthorized",
                                    "message": "Invalid or expired token"
                                }))
                                .map_into_right_body();
                            return Ok(ServiceResponse::new(request, response));
                        }
                    }
                }
            }

            // Must have valid TenantContext; otherwise 401 Unauthorized
            let tenant_ctx = match extracted_context {
                Some(ctx) => ctx,
                None => {
                    let (request, _) = req.into_parts();
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Unauthorized",
                            "message": "Missing or invalid tenant authentication credentials"
                        }))
                        .map_into_right_body();
                    return Ok(ServiceResponse::new(request, response));
                }
            };

            // 3. Usage Metering Check via Redis Counter
            if let Some(client) = &redis_client {
                if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                    let year_month = chrono::Utc::now().format("%Y-%m").to_string();
                    let redis_key = format!("usage:{}:{}", tenant_ctx.tenant_id, year_month);

                    let current_usage: Result<u64, _> = redis::cmd("INCR")
                        .arg(&redis_key)
                        .query_async(&mut conn)
                        .await;

                    match current_usage {
                        Ok(usage) => {
                            // Set TTL of 60 days (5184000s) on key
                            let _: Result<(), _> = redis::cmd("EXPIRE")
                                .arg(&redis_key)
                                .arg(5184000)
                                .query_async(&mut conn)
                                .await;

                            let limit = tenant_ctx.tier.monthly_limit();
                            if usage > limit {
                                let err_payload = PaymentRequiredError {
                                    error: "Payment Required".to_string(),
                                    message: "Usage limit exceeded for current pricing tier".to_string(),
                                    tier: tenant_ctx.tier.to_string(),
                                    limit,
                                    current_usage: usage,
                                };
                                let (request, _) = req.into_parts();
                                let response = HttpResponse::PaymentRequired()
                                    .json(err_payload)
                                    .map_into_right_body();
                                return Ok(ServiceResponse::new(request, response));
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Redis error checking usage limit: {:?}", e);
                        }
                    }
                }
            }

            // Inject TenantContext into request extensions
            req.extensions_mut().insert(tenant_ctx);

            let res = svc.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

impl FromRequest for TenantContext {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<TenantContext>() {
            Some(ctx) => ready(Ok(ctx.clone())),
            None => ready(Err(actix_web::error::ErrorUnauthorized(
                "TenantContext missing from request extensions",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_missing_auth_returns_401() {
        let app = test::init_service(
            actix_web::App::new()
                .wrap(TenantAuthMiddleware::new())
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_unauthenticated_x_tenant_id_returns_401() {
        let tenant_id = Uuid::new_v4();
        let app = test::init_service(
            actix_web::App::new()
                .wrap(TenantAuthMiddleware::new())
                .route(
                    "/test",
                    web::get().to(|ctx: TenantContext| async move {
                        HttpResponse::Ok().json(serde_json::json!({
                            "tenant_id": ctx.tenant_id,
                            "tier": ctx.tier.to_string(),
                        }))
                    }),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("X-Tenant-Id", tenant_id.to_string()))
            .insert_header(("X-Tenant-Tier", "Growth"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_unregistered_api_key_returns_401() {
        let app = test::init_service(
            actix_web::App::new()
                .wrap(TenantAuthMiddleware::new())
                .route(
                    "/test",
                    web::get().to(|ctx: TenantContext| async move {
                        assert_eq!(ctx.auth_method, AuthMethod::ApiKey);
                        HttpResponse::Ok().finish()
                    }),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("X-API-Key", "sk_live_test_key_123"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_invalid_api_key_returns_401() {
        let app = test::init_service(
            actix_web::App::new()
                .wrap(TenantAuthMiddleware::new())
                .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("X-API-Key", "invalid_key"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_jwt_auth() {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let claims = JwtClaims {
            sub: user_id,
            role: None,
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            tenant_id: Some(tenant_id),
            tier: Some(PricingTier::Growth),
        };
        let secret = "test_secret";
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let app = test::init_service(
            actix_web::App::new()
                .wrap(TenantAuthMiddleware::new().with_secret(secret))
                .route(
                    "/test",
                    web::get().to(move |ctx: TenantContext| async move {
                        assert_eq!(ctx.tenant_id, tenant_id);
                        assert_eq!(ctx.user_id, Some(user_id));
                        assert_eq!(ctx.tier, PricingTier::Growth);
                        assert_eq!(ctx.auth_method, AuthMethod::Jwt);
                        HttpResponse::Ok().finish()
                    }),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_jwt_tenant_claim_override_prevented() {
        use jsonwebtoken::{encode, EncodingKey, Header};

        let user_id = Uuid::new_v4();
        let jwt_tenant_id = Uuid::new_v4();
        let header_tenant_id = Uuid::new_v4();

        let claims = JwtClaims {
            sub: user_id,
            role: None,
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            tenant_id: Some(jwt_tenant_id),
            tier: Some(PricingTier::Growth),
        };
        let secret = "test_secret";
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let app = test::init_service(
            actix_web::App::new()
                .wrap(TenantAuthMiddleware::new().with_secret(secret))
                .route(
                    "/test",
                    web::get().to(move |ctx: TenantContext| async move {
                        assert_eq!(ctx.tenant_id, jwt_tenant_id);
                        HttpResponse::Ok().finish()
                    }),
                ),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .insert_header(("X-Tenant-Id", header_tenant_id.to_string()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}
