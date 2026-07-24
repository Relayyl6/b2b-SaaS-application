use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use platform::streams::StreamPublisher;
use uuid::Uuid;

use crate::db::PaymentRepo;
use crate::models::{
    CreatePaymentIntentRequest, PaymentEvent, PaymentIntent, PaymentStatus, PaymentWebhook,
};

pub async fn create_payment_intent(
    repo: web::Data<PaymentRepo>,
    publisher: web::Data<StreamPublisher>,
    req: web::Json<CreatePaymentIntentRequest>,
) -> impl Responder {
    match repo.create_intent(&req).await {
        Ok(intent) => {
            publish_payment_event(&publisher, "payment.initiated", &intent);
            HttpResponse::Created().json(intent)
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
    webhook: web::Json<PaymentWebhook>,
) -> impl Responder {
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

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok","service":"payments"}))
}
