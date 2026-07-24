use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use platform::streams::StreamPublisher;
use uuid::Uuid;

use crate::db::SupplierRepo;
use crate::models::{CreateSupplierRequest, Supplier, SupplierEvent, UpdateSupplierStatusRequest};

pub async fn create_supplier(
    repo: web::Data<SupplierRepo>,
    publisher: web::Data<StreamPublisher>,
    req: web::Json<CreateSupplierRequest>,
) -> impl Responder {
    match repo.create(&req).await {
        Ok(supplier) => {
            publish_supplier_event(&publisher, "supplier.created", &supplier);
            HttpResponse::Created().json(supplier)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn get_supplier(repo: web::Data<SupplierRepo>, path: web::Path<Uuid>) -> impl Responder {
    match repo.get(path.into_inner()).await {
        Ok(supplier) => HttpResponse::Ok().json(supplier),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("supplier not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn list_owner_suppliers(
    repo: web::Data<SupplierRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    match repo.list_by_owner(path.into_inner()).await {
        Ok(suppliers) => HttpResponse::Ok().json(suppliers),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn update_supplier_status(
    repo: web::Data<SupplierRepo>,
    publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateSupplierStatusRequest>,
) -> impl Responder {
    match repo
        .update_status(path.into_inner(), req.status.clone())
        .await
    {
        Ok(supplier) => {
            publish_supplier_event(&publisher, "supplier.status_updated", &supplier);
            HttpResponse::Ok().json(supplier)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("supplier not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

fn publish_supplier_event(publisher: &StreamPublisher, event_type: &str, supplier: &Supplier) {
    publisher.publish_async(
        event_type,
        SupplierEvent {
            event_type: event_type.to_string(),
            supplier_id: supplier.id,
            user_id: supplier.owner_user_id,
            owner_user_id: supplier.owner_user_id,
            status: supplier.status.clone(),
            timestamp: Utc::now(),
        },
    );
}

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok","service":"supplier-management"}))
}
