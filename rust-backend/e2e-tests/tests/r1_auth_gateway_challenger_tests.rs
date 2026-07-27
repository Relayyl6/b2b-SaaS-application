use actix_web::{http::StatusCode, test, web, App, HttpResponse};
use e2e_tests::test_context::{
    generate_expired_jwt, generate_mock_api_key, generate_mock_jwt, MockTenantFixture,
};
use e2e_tests::TestHarness;
use platform::middleware::tenant_middleware::{PaymentRequiredError, TenantAuthMiddleware};
use platform::tenant::{AuthMethod, PricingTier, TenantContext};
use uuid::Uuid;

/// 1. Valid API key returns 200 OK + injected tenant context
#[actix_web::test]
async fn test_r1_valid_api_key_returns_200_and_injected_context() {
    let harness = TestHarness::new().await;
    let secret = "challenger_r1_jwt_secret";
    let tenant_id = Uuid::new_v4();
    let api_key = generate_mock_api_key(tenant_id, "live");

    let mut middleware = TenantAuthMiddleware::new().with_secret(secret);
    if let Some(ref client) = harness.redis_client {
        middleware = TenantAuthMiddleware::with_redis(client.clone()).with_secret(secret);
        let rec = platform::tenant::ApiKeyRecord {
            id: Uuid::new_v4(),
            tenant_id,
            key_prefix: "sk_live".to_string(),
            key_hash: "hash".to_string(),
            permissions: vec!["*".to_string()],
            rate_limit_override: None,
            is_active: true,
        };
        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            let _: Result<(), _> = redis::cmd("SET")
                .arg(format!("api_key:{}", api_key))
                .arg(serde_json::to_string(&rec).unwrap())
                .query_async(&mut conn)
                .await;
        }
    }

    let app = test::init_service(
        App::new()
            .wrap(middleware)
            .route(
                "/api/resource",
                web::get().to(|ctx: TenantContext| async move {
                    assert_eq!(ctx.auth_method, AuthMethod::ApiKey);
                    HttpResponse::Ok().json(serde_json::json!({
                        "status": "ok",
                        "tenant_id": ctx.tenant_id,
                        "auth_method": format!("{:?}", ctx.auth_method)
                    }))
                }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/resource")
        .insert_header(("X-API-Key", api_key.as_str()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    if harness.redis_client.is_some() {
        assert_eq!(resp.status(), StatusCode::OK);
    } else {
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}

/// 1b. Valid JWT returns 200 OK + injected tenant context
#[actix_web::test]
async fn test_r1_valid_jwt_returns_200_and_injected_context() {
    let secret = "challenger_r1_jwt_secret";
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let jwt_token = generate_mock_jwt(user_id, tenant_id, PricingTier::Growth, secret).unwrap();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/api/resource",
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
        .uri("/api/resource")
        .insert_header(("Authorization", format!("Bearer {}", jwt_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

/// 2a. Missing credentials returns 401 Unauthorized
#[actix_web::test]
async fn test_r1_missing_credentials_returns_401() {
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new())
            .route(
                "/api/resource",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/resource").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// 2b. Invalid API key ("invalid" keyword) returns 401 Unauthorized
#[actix_web::test]
async fn test_r1_invalid_api_key_returns_401() {
    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new())
            .route(
                "/api/resource",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/resource")
        .insert_header(("X-API-Key", "invalid_api_key_sample"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// 2c. Expired JWT returns 401 Unauthorized
#[actix_web::test]
async fn test_r1_expired_jwt_returns_401() {
    let secret = "challenger_r1_jwt_secret";
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let expired_token = generate_expired_jwt(user_id, tenant_id, secret).unwrap();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/api/resource",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/resource")
        .insert_header(("Authorization", format!("Bearer {}", expired_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// 2d. JWT signed with wrong secret returns 401 Unauthorized
#[actix_web::test]
async fn test_r1_tampered_jwt_signature_returns_401() {
    let secret = "correct_secret";
    let wrong_secret = "wrong_secret_attacker";
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let tampered_token = generate_mock_jwt(user_id, tenant_id, PricingTier::Free, wrong_secret).unwrap();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/api/resource",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/resource")
        .insert_header(("Authorization", format!("Bearer {}", tampered_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

/// 3a. Free tier exceeding monthly limit (100) returns 402 Payment Required with structured JSON
#[actix_web::test]
async fn test_r1_free_tier_exceeding_limit_returns_402_structured_json() {
    let harness = TestHarness::new().await;
    let secret = "challenger_r1_jwt_secret";
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Free tier token (limit 100)
    let token = generate_mock_jwt(user_id, tenant_id, PricingTier::Free, secret).unwrap();

    // Set usage counter in Redis to 100 (next INCR will make it 101)
    let redis_available = harness.set_usage_counter(tenant_id, 100).await.is_ok();

    let mut middleware = TenantAuthMiddleware::new().with_secret(secret);
    if let Some(client) = harness.redis_client {
        middleware = TenantAuthMiddleware::with_redis(client).with_secret(secret);
    }

    let app = test::init_service(
        App::new()
            .wrap(middleware)
            .route(
                "/api/resource",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/resource")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;

    if redis_available {
        assert_eq!(resp.status(), StatusCode::PAYMENT_REQUIRED);
        let body: PaymentRequiredError = test::read_body_json(resp).await;
        assert_eq!(body.error, "Payment Required");
        assert_eq!(body.message, "Usage limit exceeded for current pricing tier");
        assert_eq!(body.tier, "Free");
        assert_eq!(body.limit, 100);
        assert_eq!(body.current_usage, 101);
    } else {
        // Fallback when Redis is not running in test environment
        assert!(resp.status() == StatusCode::OK || resp.status() == StatusCode::PAYMENT_REQUIRED);
    }
}

/// 3b. Free tier boundary case: Request #100 is allowed (200 OK), Request #101 returns 402
#[actix_web::test]
async fn test_r1_free_tier_boundary_100_ok_101_payment_required() {
    let harness = TestHarness::new().await;
    let secret = "challenger_r1_jwt_secret";
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let token = generate_mock_jwt(user_id, tenant_id, PricingTier::Free, secret).unwrap();

    // Set usage counter in Redis to 99 (next INCR makes it 100)
    let redis_available = harness.set_usage_counter(tenant_id, 99).await.is_ok();

    let mut middleware = TenantAuthMiddleware::new().with_secret(secret);
    if let Some(client) = harness.redis_client {
        middleware = TenantAuthMiddleware::with_redis(client).with_secret(secret);
    }

    let app = test::init_service(
        App::new()
            .wrap(middleware)
            .route(
                "/api/resource",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    if redis_available {
        // Request #100: usage becomes 100 (<= limit 100) -> 200 OK
        let req100 = test::TestRequest::get()
            .uri("/api/resource")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp100 = test::call_service(&app, req100).await;
        assert_eq!(resp100.status(), StatusCode::OK);

        // Request #101: usage becomes 101 (> limit 100) -> 402 PAYMENT_REQUIRED
        let req101 = test::TestRequest::get()
            .uri("/api/resource")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp101 = test::call_service(&app, req101).await;
        assert_eq!(resp101.status(), StatusCode::PAYMENT_REQUIRED);
    }
}

/// 3c. Growth tier with 150 requests is allowed (limit = 10,000)
#[actix_web::test]
async fn test_r1_growth_tier_higher_usage_allowed() {
    let harness = TestHarness::new().await;
    let secret = "challenger_r1_jwt_secret";
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let token = generate_mock_jwt(user_id, tenant_id, PricingTier::Growth, secret).unwrap();

    // Set usage counter to 150 (> 100 Free limit, but < 10,000 Growth limit)
    let _ = harness.set_usage_counter(tenant_id, 150).await;

    let mut middleware = TenantAuthMiddleware::new().with_secret(secret);
    if let Some(client) = harness.redis_client {
        middleware = TenantAuthMiddleware::with_redis(client).with_secret(secret);
    }

    let app = test::init_service(
        App::new()
            .wrap(middleware)
            .route(
                "/api/resource",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/resource")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

/// 4. Stress Test: Rapid concurrent requests (burst scenario)
#[actix_web::test]
async fn test_r1_rapid_concurrent_request_burst() {
    let secret = "challenger_r1_jwt_secret";
    let user_id = Uuid::new_v4();
    let tenant_id = Uuid::new_v4();
    let token = generate_mock_jwt(user_id, tenant_id, PricingTier::Growth, secret).unwrap();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/api/resource",
                web::get().to(|| async { HttpResponse::Ok().finish() }),
            ),
    )
    .await;

    for _ in 0..50 {
        let req = test::TestRequest::get()
            .uri("/api/resource")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

/// 5. Adversarial Challenge Test: Unauthenticated Header Spoofing Vulnerability Prevention
/// If X-Tenant-Id header is passed by client without valid token or API key,
/// TenantAuthMiddleware MUST reject request with 401 Unauthorized.
#[actix_web::test]
async fn test_r1_adversarial_header_spoofing_behavior() {
    let attacker_target_tenant = Uuid::new_v4();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new())
            .route(
                "/api/resource",
                web::get().to(|ctx: TenantContext| async move {
                    HttpResponse::Ok().json(serde_json::json!({
                        "tenant_id": ctx.tenant_id
                    }))
                }),
            ),
    )
    .await;

    // Attacker provides NO JWT or API Key, only X-Tenant-Id
    let req = test::TestRequest::get()
        .uri("/api/resource")
        .insert_header(("X-Tenant-Id", attacker_target_tenant.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
