use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use uuid::Uuid;

use crate::db::LogisticsRepo;
use crate::models::{
    CreateShipmentRequest, ListShipmentQuery, LogisticsEvent, UpdateShipmentStatusRequest,
};
use crate::publisher::RedisPublisher;
use crate::rabbit_pub::RabbitPublisher;

/// Creates a shipment and publishes logistics.shipment_created.
pub async fn create_shipment(
    repo: web::Data<LogisticsRepo>,
    redis_pub: web::Data<RedisPublisher>,
    rabbit_pub: web::Data<RabbitPublisher>,
    req: web::Json<CreateShipmentRequest>,
) -> impl Responder {
    match repo.create_shipment(&req).await {
        Ok(shipment) => {
            let event = LogisticsEvent {
                event_type: "logistics.shipment_created".into(),
                shipment_id: shipment.id,
                order_id: shipment.order_id,
                user_id: shipment.user_id,
                supplier_id: shipment.supplier_id,
                product_id: shipment.product_id,
                status: shipment.status.clone(),
                tracking_number: shipment.tracking_number.clone(),
                timestamp: Utc::now(),
            };

            redis_pub.publish_async("logistics.shipment_created", event.clone());
            rabbit_pub.publish_async(event.clone());

            HttpResponse::Created().json(shipment)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("failed to create shipment: {e}"))
        }
    }
}

/// Returns shipment details by id.
pub async fn get_shipment(repo: web::Data<LogisticsRepo>, path: web::Path<Uuid>) -> impl Responder {
    match repo.get_shipment(path.into_inner()).await {
        Ok(shipment) => HttpResponse::Ok().json(shipment),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("shipment not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

/// Returns supplier shipments using filter and pagination query fields.
/// List shipments for a supplier using filter and pagination query parameters.
///
/// Returns an HTTP response: `200 OK` with the matching shipments as JSON on success, or
/// `500 Internal Server Error` with a `db error` message on repository failure.
///
/// # Examples
///
/// ```
/// use actix_web::web;
/// use uuid::Uuid;
///
/// // Construct a supplier id and query parameters (fill fields as appropriate).
/// let supplier_id = Uuid::new_v4();
/// let query = web::Query::from(ListShipmentQuery { /* set filter/pagination fields */ });
/// // In an async test, call the handler with web::Path and web::Query wrappers:
/// // let resp = list_supplier_shipments(repo_data, web::Path::from(supplier_id), query).await;
/// ```
pub async fn list_supplier_shipments(
    repo: web::Data<LogisticsRepo>,
    path: web::Path<Uuid>,
    query: web::Query<ListShipmentQuery>,
) -> impl Responder {
    match repo
        .list_supplier_shipments(path.into_inner(), &query.into_inner())
        .await
    {
        Ok(shipments) => HttpResponse::Ok().json(shipments),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

/// Updates shipment status and publishes logistics.shipment_updated.
pub async fn update_status(
    repo: web::Data<LogisticsRepo>,
    redis_pub: web::Data<RedisPublisher>,
    rabbit_pub: web::Data<RabbitPublisher>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateShipmentStatusRequest>,
) -> impl Responder {
    match repo.update_status(path.into_inner(), &req).await {
        Ok(shipment) => {
            let event = LogisticsEvent {
                event_type: "logistics.shipment_updated".into(),
                shipment_id: shipment.id,
                order_id: shipment.order_id,
                user_id: shipment.user_id,
                supplier_id: shipment.supplier_id,
                product_id: shipment.product_id,
                status: shipment.status.clone(),
                tracking_number: shipment.tracking_number.clone(),
                timestamp: Utc::now(),
            };

            redis_pub.publish_async("logistics.shipment_updated", event.clone());
            rabbit_pub.publish_async(event.clone());

            HttpResponse::Ok().json(shipment)
        }
        Err(sqlx::Error::Protocol(message))
            if message.to_string().contains("invalid status transition") =>
        {
            HttpResponse::BadRequest().body(message.to_string())
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("shipment not found"),
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("failed to update shipment: {e}"))
        }
    }
}
