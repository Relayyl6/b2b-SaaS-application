use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use platform::streams::StreamPublisher;
use uuid::Uuid;

use crate::db::PaymentRepo;
use crate::models::{
    CreatePaymentIntentRequest, PaymentEvent, PaymentIntent, PaymentStatus, PaymentWebhook,
};
use crate::stripe::StripeClient;

pub async fn create_payment_intent(
    repo: web::Data<PaymentRepo>,
    publisher: web::Data<StreamPublisher>,
    mut req: web::Json<CreatePaymentIntentRequest>,
) -> impl Responder {
    let stripe_client = StripeClient::new();
    let amount_cents = req.amount;
    let currency = req.currency.clone().unwrap_or_else(|| "usd".to_string());

    req.provider = Some("stripe".to_string());

    // 1. Verify local DB constraints by creating the intent first
    let intent = match repo.create_intent(&req).await {
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

    match repo.update_provider_reference(intent.id, &stripe_res.id, &meta).await {
        Ok(updated_intent) => {
            publish_payment_event(&publisher, "payment.initiated", &updated_intent);
            HttpResponse::Created().json(updated_intent)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn get_payment_intent(
    repo: web::Data<PaymentRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    match repo.get(path.into_inner()).await {
        Ok(intent) => HttpResponse::Ok().json(intent),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("payment intent not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn mark_payment_succeeded(
    repo: web::Data<PaymentRepo>,
    publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    update_status(repo, publisher, path.into_inner(), PaymentStatus::Succeeded).await
}

pub async fn mark_payment_failed(
    repo: web::Data<PaymentRepo>,
    publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    update_status(repo, publisher, path.into_inner(), PaymentStatus::Failed).await
}

pub async fn payment_webhook(
    repo: web::Data<PaymentRepo>,
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

    match repo.apply_webhook(&webhook).await {
        Ok(intent) => {
            let event_type = event_type_for_status(&intent.status);
            publish_payment_event(&publisher, event_type, &intent);
            HttpResponse::Ok().json(intent)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("payment intent not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

async fn update_status(
    repo: web::Data<PaymentRepo>,
    publisher: web::Data<StreamPublisher>,
    id: Uuid,
    status: PaymentStatus,
) -> HttpResponse {
    match repo.update_status(id, status).await {
        Ok(intent) => {
            let event_type = event_type_for_status(&intent.status);
            publish_payment_event(&publisher, event_type, &intent);
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

fn publish_payment_event(publisher: &StreamPublisher, event_type: &str, intent: &PaymentIntent) {
    publisher.publish_async(
        event_type,
        PaymentEvent {
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

pub async fn refund_payment_endpoint(
    repo: web::Data<PaymentRepo>,
    publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let stripe_client = StripeClient::new();
    let id = path.into_inner();
    let intent = match repo.get(id).await {
        Ok(i) => i,
        Err(_) => return HttpResponse::NotFound().body("payment intent not found"),
    };

    if let Some(stripe_id) = &intent.provider_reference {
        match stripe_client.refund_payment(stripe_id, None, Some(&id.to_string())).await {
            Ok(_) => update_status(repo, publisher, id, PaymentStatus::Refunded).await,
            Err(e) => HttpResponse::InternalServerError().body(format!("Stripe error: {e}")),
        }
    } else {
        HttpResponse::BadRequest().body("No provider reference found")
    }
}

pub async fn transfer_payment_endpoint(
    repo: web::Data<PaymentRepo>,
    _publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let stripe_client = StripeClient::new();
    let id = path.into_inner();
    let intent = match repo.get(id).await {
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

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok","service":"payments"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use sqlx::PgPool;
    use platform::streams::StreamPublisher;

    // We use sqlx::test to get a real DB pool
    #[sqlx::test]
    async fn test_create_payment_intent_handler(pool: PgPool) {
        let repo = web::Data::new(PaymentRepo::new(pool));
        
        let publisher = web::Data::new(StreamPublisher::noop());

        let app = test::init_service(
            App::new()
                .app_data(repo.clone())
                .app_data(publisher.clone())
                .route("/intents", web::post().to(create_payment_intent))
        ).await;

        let req_body = CreatePaymentIntentRequest {
            idempotency_key: "handler_idemp_key".to_string(),
            order_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            supplier_id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            quantity: 1,
            amount: 1500,
            currency: Some("usd".to_string()),
            provider: None,
            metadata: None,
        };

        let req = test::TestRequest::post()
            .uri("/intents")
            .set_json(&req_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "Expected success but got {}", resp.status());
    }

    #[sqlx::test]
    async fn test_health_handler(pool: PgPool) {
        let app = test::init_service(
            App::new().route("/health", web::get().to(health))
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}
