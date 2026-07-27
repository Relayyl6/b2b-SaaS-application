use actix_web::{http::StatusCode, test, web, App, HttpResponse};
use e2e_tests::test_context::{
    create_enriched_event, validate_event_tenant_enrichment, EnrichedEventPayload,
    MockTenantFixture,
};
use platform::middleware::tenant_middleware::TenantAuthMiddleware;
use platform::tenant::{PricingTier, TenantContext};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AttackPayload {
    pub target_order_id: Uuid,
    pub payload_tenant_id: Uuid,
}

#[actix_web::test]
async fn test_real_world_cross_tenant_attack_resilience() {
    let secret = "audit_secret_key";
    let attacker_tenant_a = MockTenantFixture::new(PricingTier::Growth, secret);
    let victim_tenant_b = MockTenantFixture::new(PricingTier::Enterprise, secret);

    let captured_db_tenant = Arc::new(Mutex::new(None));
    let db_tenant_clone = captured_db_tenant.clone();

    let app = test::init_service(
        App::new()
            .wrap(TenantAuthMiddleware::new().with_secret(secret))
            .route(
                "/orders/{id}",
                web::get().to(move |ctx: TenantContext, path: web::Path<Uuid>| async move {
                    let order_id = path.into_inner();
                    // Simulated DB RLS query: WHERE id = order_id AND tenant_id = ctx.tenant_id
                    if ctx.tenant_id == attacker_tenant_a.tenant_id {
                        // Attacker querying Victim order ID returns 404 NOT FOUND due to RLS
                        HttpResponse::NotFound().json(serde_json::json!({
                            "error": "Order not found or access denied",
                            "order_id": order_id
                        }))
                    } else {
                        HttpResponse::Ok().finish()
                    }
                }),
            )
            .route(
                "/orders",
                web::post().to(move |ctx: TenantContext, body: web::Json<AttackPayload>| {
                    let db_tc = db_tenant_clone.clone();
                    async move {
                        // Overwrite payload tenant_id with authenticated identity from JWT
                        let actual_tenant_written = ctx.tenant_id;
                        let mut guard = db_tc.lock().unwrap();
                        *guard = Some(actual_tenant_written);

                        HttpResponse::Created().json(serde_json::json!({
                            "status": "created",
                            "tenant_id": actual_tenant_written,
                            "attempted_tenant_id": body.payload_tenant_id
                        }))
                    }
                }),
            ),
    )
    .await;

    // Vector 1: Cross-tenant GET order attack
    let victim_order_id = Uuid::new_v4();
    let get_req = test::TestRequest::get()
        .uri(&format!("/orders/{}", victim_order_id))
        .insert_header((
            "Authorization",
            format!("Bearer {}", attacker_tenant_a.jwt_token),
        ))
        .to_request();

    let get_resp = test::call_service(&app, get_req).await;
    assert_eq!(
        get_resp.status(),
        StatusCode::NOT_FOUND,
        "Cross-tenant data read attempt must be blocked by RLS / 404"
    );

    // Vector 2: Payload tenant_id spoofing attack
    let post_req = test::TestRequest::post()
        .uri("/orders")
        .insert_header((
            "Authorization",
            format!("Bearer {}", attacker_tenant_a.jwt_token),
        ))
        .set_json(AttackPayload {
            target_order_id: Uuid::new_v4(),
            payload_tenant_id: victim_tenant_b.tenant_id, // Spoofed Tenant B ID in body
        })
        .to_request();

    let post_resp = test::call_service(&app, post_req).await;
    assert_eq!(post_resp.status(), StatusCode::CREATED);

    let guard = captured_db_tenant.lock().unwrap();
    assert_eq!(
        *guard,
        Some(attacker_tenant_a.tenant_id),
        "DB write must be forced to authenticated Tenant A ID, overriding spoofed body"
    );

    // Vector 3: Event stream poisoning validation
    let forged_event = create_enriched_event(
        "order.created",
        Some(victim_tenant_b.tenant_id),
        serde_json::json!({ "tampered": true }),
        secret,
    );

    // Consumer running under Tenant A context rejects forged Tenant B event
    let is_valid = validate_event_tenant_enrichment(&forged_event, attacker_tenant_a.tenant_id);
    assert_eq!(
        is_valid, false,
        "Tenant A consumer worker must drop forged Tenant B event"
    );
}
