use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use platform::{streams::StreamPublisher, tenant::TenantContext, db_router::DynamicPoolRouter};
use uuid::Uuid;

use crate::db::PaymentRepo;
use crate::models::{
    CreatePaymentIntentRequest, PaymentEvent, PaymentIntent, PaymentStatus, PaymentWebhook,
};
use crate::stripe::StripeClient;

#[utoipa::path(
    post,
    path = "/payments/intents",
    request_body = CreatePaymentIntentRequest,
    responses(
        (status = 201, description = "Payment intent created", body = PaymentIntent),
        (status = 500, description = "Internal server error")
    ),
    security(("BearerAuth" = []), ("ApiKeyAuth" = []))
)]
pub async fn create_payment_intent(
    tenant: actix_web::web::ReqData<TenantContext>,
    db_router: actix_web::web::Data<DynamicPoolRouter>,
    publisher: web::Data<StreamPublisher>,
    mut req: web::Json<CreatePaymentIntentRequest>,
) -> impl Responder {
    let stripe_client = StripeClient::new();
    let amount_cents = req.amount;
    let currency = req.currency.clone().unwrap_or_else(|| "usd".to_string());

    req.provider = Some("stripe".to_string());

let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    let intent = match PaymentRepo::create_intent(&mut *tx, &tenant.tenant_id, &req).await {
        Ok(i) => i,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };

    // 2. Call Stripe
    let stripe_res = match stripe_client.create_payment_intent(amount_cents, &currency, req.metadata.clone(), &req.idempotency_key).await {
        Ok(res) => res,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    // 3. Update local DB with Stripe ID
    let mut meta = req.metadata.clone().unwrap_or_else(|| serde_json::json!({}));
    if let Some(obj) = meta.as_object_mut() {
        obj.insert("client_secret".to_string(), serde_json::Value::String(stripe_res.client_secret));
        obj.insert("stripe_id".to_string(), serde_json::Value::String(stripe_res.id.clone()));
    }

    match PaymentRepo::update_provider_reference(&mut *tx, intent.id, &stripe_res.id, &meta).await {
        Ok(updated_intent) => {
            tx.commit().await.unwrap();
            publish_payment_event(&publisher, tenant.tenant_id, "payment.initiated", &updated_intent);
            HttpResponse::Created().json(updated_intent)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    get,
    path = "/payments/intents/{id}",
    params(
        ("id" = Uuid, Path, description = "Payment intent id")
    ),
    responses(
        (status = 200, description = "Payment intent found", body = PaymentIntent),
        (status = 404, description = "Payment intent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("BearerAuth" = []), ("ApiKeyAuth" = []))
)]
pub async fn get_payment_intent(
    tenant: actix_web::web::ReqData<TenantContext>,
    db_router: actix_web::web::Data<DynamicPoolRouter>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match PaymentRepo::get(&mut *tx, path.into_inner()).await {
        Ok(intent) => { tx.commit().await.unwrap(); HttpResponse::Ok().json(intent) },
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("payment intent not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    post,
    path = "/payments/intents/{id}/succeed",
    params(
        ("id" = Uuid, Path, description = "Payment intent id")
    ),
    responses(
        (status = 200, description = "Payment intent updated", body = PaymentIntent),
        (status = 404, description = "Payment intent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("BearerAuth" = []), ("ApiKeyAuth" = []))
)]
pub async fn mark_payment_succeeded(
    tenant: actix_web::web::ReqData<TenantContext>,
    db_router: actix_web::web::Data<DynamicPoolRouter>,
    publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    update_status(tenant, db_router, publisher, path.into_inner(), PaymentStatus::Succeeded).await
}

#[utoipa::path(
    post,
    path = "/payments/intents/{id}/fail",
    params(
        ("id" = Uuid, Path, description = "Payment intent id")
    ),
    responses(
        (status = 200, description = "Payment intent updated", body = PaymentIntent),
        (status = 404, description = "Payment intent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("BearerAuth" = []), ("ApiKeyAuth" = []))
)]
pub async fn mark_payment_failed(
    tenant: actix_web::web::ReqData<TenantContext>,
    db_router: actix_web::web::Data<DynamicPoolRouter>,
    publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    update_status(tenant, db_router, publisher, path.into_inner(), PaymentStatus::Failed).await
}

#[utoipa::path(
    post,
    path = "/payments/webhooks",
    request_body = PaymentWebhook,
    responses(
        (status = 200, description = "Webhook processed", body = PaymentIntent),
        (status = 400, description = "Invalid payload or signature"),
        (status = 404, description = "Payment intent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("BearerAuth" = []), ("ApiKeyAuth" = []))
)]
pub async fn payment_webhook(
    db_router: actix_web::web::Data<DynamicPoolRouter>,
    publisher: web::Data<StreamPublisher>,
    req: actix_web::HttpRequest,
    body: web::Bytes,
) -> impl Responder {
    let stripe_client = StripeClient::new();
    let signature = req.headers().get("Stripe-Signature").and_then(|v| v.to_str().ok()).unwrap_or("");
    let payload_str = String::from_utf8_lossy(&body);

    if let Err(e) = stripe_client.verify_webhook_signature(&payload_str, signature) {
        return HttpResponse::BadRequest().body(format!("Invalid signature: {e}"));
    }

    let Ok(webhook) = serde_json::from_str::<PaymentWebhook>(&payload_str) else {
        return HttpResponse::BadRequest().body("Invalid webhook payload");
    };

    let pool = db_router.shared_pool();

    match PaymentRepo::apply_webhook(pool, &webhook).await {
        Ok(intent) => {
            let event_type = event_type_for_status(&intent.status);
            publish_payment_event(&publisher, intent.tenant_id, event_type, &intent);
            HttpResponse::Ok().json(intent)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("payment intent not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

async fn update_status(
    tenant: actix_web::web::ReqData<TenantContext>,
    db_router: actix_web::web::Data<DynamicPoolRouter>,
    publisher: web::Data<StreamPublisher>,
    id: Uuid,
    status: PaymentStatus,
) -> HttpResponse {
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    match PaymentRepo::update_status(&mut *tx, id, status).await {
        Ok(intent) => {
            tx.commit().await.unwrap();
            let event_type = event_type_for_status(&intent.status);
            publish_payment_event(&publisher, tenant.tenant_id, event_type, &intent);
            HttpResponse::Ok().json(intent)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("payment intent not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

fn event_type_for_status(status: &PaymentStatus) -> &'static str {
    match status {
        PaymentStatus::Succeeded => "payment.success",
        PaymentStatus::Failed => "payment.failed",
        PaymentStatus::Cancelled => "payment.cancelled",
        PaymentStatus::Refunded => "payment.refunded",
        PaymentStatus::Processing => "payment.processing",
        PaymentStatus::RequiresPaymentMethod => "payment.initiated",
    }
}

fn publish_payment_event(publisher: &StreamPublisher, tenant_id: Uuid, event_type: &str, intent: &PaymentIntent) {
    publisher.publish_async(
        event_type,
        PaymentEvent {
            tenant_id,
            event_type: event_type.to_string(),
            payment_id: intent.id,
            order_id: intent.order_id,
            user_id: intent.user_id,
            supplier_id: intent.supplier_id,
            product_id: intent.product_id,
            quantity: intent.quantity,
            amount: intent.amount,
            currency: intent.currency.clone(),
            provider: intent.provider.clone(),
            provider_reference: intent.provider_reference.clone(),
            timestamp: Utc::now(),
        },
    );
}

#[utoipa::path(
    post,
    path = "/payments/intents/{id}/refund",
    params(
        ("id" = Uuid, Path, description = "Payment intent id")
    ),
    responses(
        (status = 200, description = "Payment refunded", body = PaymentIntent),
        (status = 400, description = "Bad Request"),
        (status = 404, description = "Payment intent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("BearerAuth" = []), ("ApiKeyAuth" = []))
)]
pub async fn refund_payment_endpoint(
    tenant: actix_web::web::ReqData<TenantContext>,
    db_router: actix_web::web::Data<DynamicPoolRouter>,
    publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let stripe_client = StripeClient::new();
    let id = path.into_inner();
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    let intent = match PaymentRepo::get(&mut *tx, id).await {
        Ok(i) => i,
        Err(_) => return HttpResponse::NotFound().body("payment intent not found"),
    };

    if let Some(stripe_id) = &intent.provider_reference {
        match stripe_client.refund_payment(stripe_id, None, Some(&id.to_string())).await {
            Ok(_) => { tx.commit().await.unwrap(); update_status(tenant, db_router, publisher, id, PaymentStatus::Refunded).await },
            Err(e) => HttpResponse::InternalServerError().body(format!("Stripe error: {e}")),
        }
    } else {
        HttpResponse::BadRequest().body("No provider reference found")
    }
}

#[utoipa::path(
    post,
    path = "/payments/intents/{id}/transfer",
    params(
        ("id" = Uuid, Path, description = "Payment intent id")
    ),
    responses(
        (status = 200, description = "Payment transferred"),
        (status = 404, description = "Payment intent not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("BearerAuth" = []), ("ApiKeyAuth" = []))
)]
pub async fn transfer_payment_endpoint(
    tenant: actix_web::web::ReqData<TenantContext>,
    db_router: actix_web::web::Data<DynamicPoolRouter>,
    _publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let stripe_client = StripeClient::new();
    let id = path.into_inner();
    let pool = db_router.get_pool(&tenant).await.unwrap();
    let mut tx = pool.begin().await.unwrap();
    tenant.apply_rls(&mut *tx).await.unwrap();

    let intent = match PaymentRepo::get(&mut *tx, id).await {
        Ok(i) => i,
        Err(_) => return HttpResponse::NotFound().body("payment intent not found"),
    };

    // Assume 5% fee
    let amount_cents = intent.amount;
    let platform_fee = (amount_cents as f64 * 0.05) as i64;
    let payout_amount = amount_cents - platform_fee;

    // Mock supplier stripe account id
    let stripe_account_id = format!("acct_mock_{}", intent.supplier_id);

    match stripe_client.transfer_to_supplier(payout_amount, &intent.currency, &stripe_account_id, Some(&id.to_string())).await {
        Ok(tr_id) => HttpResponse::Ok().json(serde_json::json!({
            "transfer_id": tr_id,
            "payout_amount_cents": payout_amount
        })),
        Err(e) => HttpResponse::InternalServerError().body(format!("Stripe error: {e}")),
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check OK")
    )
)]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok","service":"payments"}))
}


