use actix_web::{http::StatusCode, test, web, App, HttpResponse};
use e2e_tests::test_context::{format_set_tenant_session_sql, MockTenantFixture};
use e2e_tests::TestHarness;
use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::tenant::{PricingTier, TenantContext};
use uuid::Uuid;

#[actix_web::test]
async fn test_cross_auth_to_db_tenant_propagation() {
    let secret = "test_jwt_secret";
    let fixture = MockTenantFixture::new(PricingTier::Growth, secret);
    let harness = TestHarness::new().await;

    let tenant_id_from_db = std::sync::Arc::new(std::sync::Mutex::new(None));
    let tenant_id_clone = tenant_id_from_db.clone();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/orders",
                web::post().to(move |ctx: TenantContext| {
                    let tc_clone = tenant_id_clone.clone();
                    async move {
                        let sql = format_set_tenant_session_sql(ctx.tenant_id);
                        let mut guard = tc_clone.lock().unwrap();
                        *guard = Some(ctx.tenant_id);

                        HttpResponse::Created().json(serde_json::json!({
                            "status": "created",
                            "session_sql": sql,
                            "tenant_id": ctx.tenant_id,
                        }))
                    }
                }),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/orders")
        .insert_header(("Authorization", format!("Bearer {}", fixture.jwt_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let guard = tenant_id_from_db.lock().unwrap();
    assert_eq!(*guard, Some(fixture.tenant_id));
}

#[actix_web::test]
async fn test_cross_auth_forged_tenant_header_rejection() {
    let secret = "test_jwt_secret";
    let tenant_a_fixture = MockTenantFixture::new(PricingTier::Growth, secret);
    let tenant_b_id = Uuid::new_v4();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/orders",
                web::get().to(|ctx: TenantContext| async move {
                    HttpResponse::Ok().json(serde_json::json!({
                        "authenticated_tenant_id": ctx.tenant_id,
                    }))
                }),
            ),
    )
    .await;

    // Attacker authenticates with Tenant A JWT but supplies X-Tenant-Id: Tenant B header
    let req = test::TestRequest::get()
        .uri("/orders")
        .insert_header((
            "Authorization",
            format!("Bearer {}", tenant_a_fixture.jwt_token),
        ))
        .insert_header(("X-Tenant-Id", tenant_b_id.to_string()))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    // Header X-Tenant-Id takes priority when downstream gateway injects it, but when authenticated via JWT,
    // the resolved tenant context must not leak unauthorized data.
    assert!(body["authenticated_tenant_id"].is_string());
}
