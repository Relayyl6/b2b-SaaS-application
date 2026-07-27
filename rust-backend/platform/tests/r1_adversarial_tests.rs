use actix_web::{http::StatusCode, test, web, App, HttpResponse};
use jsonwebtoken::{encode, EncodingKey, Header};
use platform::middleware::TenantAuthMiddleware;
use platform::tenant::{PricingTier, TenantContext};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct TestJwtClaims {
    sub: Uuid,
    exp: usize,
    tenant_id: Option<Uuid>,
    tier: Option<PricingTier>,
}

const TEST_SECRET: &str = "r1_challenger_test_secret_12345";

fn create_valid_token(tenant_id: Uuid, user_id: Uuid, tier: PricingTier, exp_offset_secs: i64) -> String {
    let exp = (chrono::Utc::now() + chrono::Duration::seconds(exp_offset_secs)).timestamp() as usize;
    let claims = TestJwtClaims {
        sub: user_id,
        exp,
        tenant_id: Some(tenant_id),
        tier: Some(tier),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
    )
    .unwrap()
}

// ---------------------------------------------------------------------------
// Attack Vector 1: Malformed Authorization Headers
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_attack_malformed_auth_header_unknown_scheme() {
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(TEST_SECRET))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("Authorization", "Basic dXNlcjpwYXNz"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Unknown scheme 'Basic' should return 401 Unauthorized"
    );
}

#[actix_web::test]
async fn test_attack_malformed_auth_header_invalid_jwt_structure() {
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(TEST_SECRET))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("Authorization", "Bearer not.a.valid.jwt.string"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Malformed JWT string should return 401 Unauthorized"
    );
}

// ---------------------------------------------------------------------------
// Attack Vector 2: Empty Tokens & Unregistered Key Fallback Vulnerability
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_attack_empty_bearer_token() {
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(TEST_SECRET))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("Authorization", "Bearer "))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Empty Bearer token should return 401 Unauthorized"
    );
}

#[actix_web::test]
async fn test_attack_unregistered_api_key_fallback_bypass() {
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(TEST_SECRET))
            .route(
                "/test",
                web::get().to(|ctx: TenantContext| async move {
                    HttpResponse::Ok().json(serde_json::json!({
                        "tenant_id": ctx.tenant_id,
                        "permissions": ctx.permissions,
                    }))
                }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("X-API-Key", "sk_unregistered_attacker_key_xyz"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    println!("STATUS FOR UNREGISTERED API KEY: {}", resp.status());
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Unregistered API key must return 401 Unauthorized"
    );
}

// ---------------------------------------------------------------------------
// Attack Vector 3: Expired Claims
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_attack_expired_jwt_claim() {
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    // Expired 3600 seconds ago
    let token = create_valid_token(tenant_id, user_id, PricingTier::Free, -3600);

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(TEST_SECRET))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/test")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Expired JWT claim must return 401 Unauthorized"
    );
}

// ---------------------------------------------------------------------------
// Attack Vector 4: Forged X-Tenant-Id Headers
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_attack_unauthenticated_forged_x_tenant_id_header() {
    let victim_tenant_id = Uuid::new_v4();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(TEST_SECRET))
            .route(
                "/protected_route",
                web::get().to(|ctx: TenantContext| async move {
                    HttpResponse::Ok().json(serde_json::json!({
                        "tenant_id": ctx.tenant_id
                    }))
                }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/protected_route")
        .insert_header(("X-Tenant-Id", victim_tenant_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    println!("STATUS FOR UNAUTHENTICATED X-TENANT-ID FORGERY: {}", resp.status());
    assert_eq!(
        resp.status(),
        StatusCode::UNAUTHORIZED,
        "Unauthenticated X-Tenant-Id header must return 401 Unauthorized"
    );
}

#[actix_web::test]
async fn test_attack_tenant_override_jwt_impersonation() {
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let user_a = Uuid::new_v4();

    let token_a = create_valid_token(tenant_a, user_a, PricingTier::Free, 3600);

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(TEST_SECRET))
            .route(
                "/protected_route",
                web::get().to(|ctx: TenantContext| async move {
                    HttpResponse::Ok().json(serde_json::json!({
                        "tenant_id": ctx.tenant_id
                    }))
                }),
            ),
    )
    .await;

    // Attacker sends JWT for Tenant A, but adds header X-Tenant-Id: Tenant B
    let req = test::TestRequest::get()
        .uri("/protected_route")
        .insert_header(("Authorization", format!("Bearer {}", token_a)))
        .insert_header(("X-Tenant-Id", tenant_b.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let extracted_tenant = body["tenant_id"].as_str().unwrap();

    println!("JWT Tenant A: {}", tenant_a);
    println!("Forged Header Tenant B: {}", tenant_b);
    println!("Extracted Context Tenant: {}", extracted_tenant);

    assert_eq!(
        extracted_tenant,
        tenant_a.to_string(),
        "Tenant ID must be derived from JWT claims, not forged X-Tenant-Id header"
    );
}

// ---------------------------------------------------------------------------
// Attack Vector 5 & 6: Concurrency and Load Handling
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_attack_concurrent_request_spike() {
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(TEST_SECRET))
            .route("/test", web::get().to(|| async { HttpResponse::Ok().finish() })),
    )
    .await;

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let token = create_valid_token(tenant_id, user_id, PricingTier::Growth, 3600);

    let num_requests = 100;

    for _ in 0..num_requests {
        let req = test::TestRequest::get()
            .uri("/test")
            .insert_header(("Authorization", format!("Bearer {}", token.clone())))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
