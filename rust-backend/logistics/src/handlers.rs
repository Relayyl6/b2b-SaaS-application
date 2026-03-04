use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use uuid::Uuid;

use crate::db::LogisticsRepo;
use crate::models::{
    CreateShipmentRequest, ListShipmentQuery, LogisticsEvent, UpdateShipmentStatusRequest,
};
use crate::publisher::RedisPublisher;

pub async fn create_shipment(
    repo: web::Data<LogisticsRepo>,
    redis_pub: web::Data<RedisPublisher>,
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

            if let Err(e) = redis_pub
                .publish("logistics.shipment_created", &event)
                .await
            {
                eprintln!("Redis publish error logistics.shipment_created: {e:?}");
            }

            HttpResponse::Created().json(shipment)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("failed to create shipment: {e}"))
        }
    }
}

pub async fn get_shipment(repo: web::Data<LogisticsRepo>, path: web::Path<Uuid>) -> impl Responder {
    match repo.get_shipment(path.into_inner()).await {
        Ok(shipment) => HttpResponse::Ok().json(shipment),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("shipment not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

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

pub async fn update_status(
    repo: web::Data<LogisticsRepo>,
    redis_pub: web::Data<RedisPublisher>,
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

            if let Err(e) = redis_pub
                .publish("logistics.shipment_updated", &event)
                .await
            {
                eprintln!("Redis publish error logistics.shipment_updated: {e:?}");
            }

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
