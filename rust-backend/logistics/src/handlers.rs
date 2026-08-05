use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use uuid::Uuid;

use crate::db::LogisticsRepo;
use crate::models::{
    CreateShipmentRequest, ListShipmentQuery, LogisticsEvent, Shipment, UpdateShipmentStatusRequest,
};
use crate::publisher::RedisPublisher;
use crate::rabbit_pub::RabbitPublisher;

/// Creates a shipment and publishes logistics.shipment_created.
#[utoipa::path(
    post,
    path = "/shipments",
    request_body = CreateShipmentRequest,
    responses((status = 201, description = "Shipment created", body = Shipment))
)]
pub async fn create_shipment(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    repo: web::Data<LogisticsRepo>,
    redis_pub: web::Data<RedisPublisher>,
    rabbit_pub: web::Data<RabbitPublisher>,
    req: web::Json<CreateShipmentRequest>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(format!("tx begin error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match repo.create_shipment(&mut *tx, tenant.tenant_id, &req).await {
        Ok(shipment) => {
            if let Err(e) = tx.commit().await {
                return HttpResponse::InternalServerError().body(format!("tx commit error: {e}"));
            }
            let event = LogisticsEvent {
                tenant_id: tenant.tenant_id,
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
#[utoipa::path(
    get,
    path = "/shipments/{shipment_id}",
    params(("shipment_id" = Uuid, Path, description = "Shipment ID")),
    responses((status = 200, description = "Shipment found", body = Shipment))
)]
pub async fn get_shipment(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    repo: web::Data<LogisticsRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(format!("tx begin error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match repo.get_shipment(&mut *tx, path.into_inner()).await {
        Ok(shipment) => {
            let _ = tx.commit().await;
            HttpResponse::Ok().json(shipment)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("shipment not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

/// Returns supplier shipments using filter and pagination query fields.
#[utoipa::path(
    get,
    path = "/shipments/supplier/{supplier_id}",
    params(("supplier_id" = Uuid, Path, description = "Supplier ID"), ListShipmentQuery),
    responses((status = 200, description = "Shipments found", body = Vec<Shipment>))
)]
pub async fn list_supplier_shipments(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    repo: web::Data<LogisticsRepo>,
    path: web::Path<Uuid>,
    query: web::Query<ListShipmentQuery>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(format!("tx begin error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match repo
        .list_supplier_shipments(&mut *tx, path.into_inner(), &query.into_inner())
        .await
    {
        Ok(shipments) => {
            let _ = tx.commit().await;
            HttpResponse::Ok().json(shipments)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

/// Updates shipment status and publishes logistics.shipment_updated.
#[utoipa::path(
    put,
    path = "/shipments/{shipment_id}/status",
    params(("shipment_id" = Uuid, Path, description = "Shipment ID")),
    request_body = UpdateShipmentStatusRequest,
    responses((status = 200, description = "Shipment updated", body = Shipment))
)]
pub async fn update_status(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    repo: web::Data<LogisticsRepo>,
    redis_pub: web::Data<RedisPublisher>,
    rabbit_pub: web::Data<RabbitPublisher>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateShipmentStatusRequest>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(format!("tx begin error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match repo.update_status(&mut *tx, path.into_inner(), &req).await {
        Ok(shipment) => {
            if let Err(e) = tx.commit().await {
                return HttpResponse::InternalServerError().body(format!("tx commit error: {e}"));
            }
            let event = LogisticsEvent {
                tenant_id: tenant.tenant_id,
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

/// Cancels an active shipment by order id and publishes logistics.shipment_cancelled.
#[utoipa::path(
    put,
    path = "/shipments/order/{order_id}/cancel",
    params(("order_id" = Uuid, Path, description = "Order ID")),
    responses((status = 200, description = "Shipment cancelled", body = Shipment))
)]
pub async fn cancel_shipment_by_order(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    repo: web::Data<LogisticsRepo>,
    redis_pub: web::Data<RedisPublisher>,
    rabbit_pub: web::Data<RabbitPublisher>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(format!("tx begin error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match repo.cancel_by_order_id(&mut *tx, path.into_inner()).await {
        Ok(shipment) => {
            if let Err(e) = tx.commit().await {
                return HttpResponse::InternalServerError().body(format!("tx commit error: {e}"));
            }
            let event = LogisticsEvent {
                tenant_id: tenant.tenant_id,
                event_type: "logistics.shipment_cancelled".into(),
                shipment_id: shipment.id,
                order_id: shipment.order_id,
                user_id: shipment.user_id,
                supplier_id: shipment.supplier_id,
                product_id: shipment.product_id,
                status: shipment.status.clone(),
                tracking_number: shipment.tracking_number.clone(),
                timestamp: Utc::now(),
            };

            redis_pub.publish_async("logistics.shipment_cancelled", event.clone());
            rabbit_pub.publish_async(event.clone());

            HttpResponse::Ok().json(shipment)
        }
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().body("active shipment for order not found")
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("failed to cancel shipment: {e}"))
        }
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Health check"))
)]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok", "service": "logistics" }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health_handler() {
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = health().await;
    }
}
