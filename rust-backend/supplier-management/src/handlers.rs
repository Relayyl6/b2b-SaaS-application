use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use platform::streams::StreamPublisher;
use uuid::Uuid;

use crate::db::SupplierRepo;
use crate::models::{CreateSupplierRequest, Supplier, SupplierEvent, UpdateSupplierStatusRequest, UpdateSupplierRequest};
use actix_web::HttpRequest;

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
    http_req: HttpRequest,
) -> impl Responder {
    let owner_user_id = match http_req.headers().get("X-User-Id") {
        Some(h) => match Uuid::parse_str(h.to_str().unwrap_or("")) {
            Ok(uuid) => uuid,
            Err(_) => return HttpResponse::BadRequest().body("Invalid X-User-Id header"),
        },
        None => return HttpResponse::Unauthorized().body("Missing X-User-Id header"),
    };

    match repo
        .update_status(path.into_inner(), owner_user_id, req.status.clone())
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
            tenant_id: Some(supplier.id),
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

pub async fn update_supplier(
    repo: web::Data<SupplierRepo>,
    publisher: web::Data<StreamPublisher>,
    path: web::Path<Uuid>,
    req: web::Json<UpdateSupplierRequest>,
    http_req: HttpRequest,
) -> impl Responder {
    let owner_user_id = match http_req.headers().get("X-User-Id") {
        Some(h) => match Uuid::parse_str(h.to_str().unwrap_or("")) {
            Ok(uuid) => uuid,
            Err(_) => return HttpResponse::BadRequest().body("Invalid X-User-Id header"),
        },
        None => return HttpResponse::Unauthorized().body("Missing X-User-Id header"),
    };

    match repo
        .update_supplier(path.into_inner(), owner_user_id, &req)
        .await
    {
        Ok(supplier) => {
            publish_supplier_event(&publisher, "supplier.updated", &supplier);
            HttpResponse::Ok().json(supplier)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("supplier not found or not owned by user"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web::Data};
    use sqlx::postgres::PgPoolOptions;

    fn dummy_repo() -> Data<SupplierRepo> {
        let pool = PgPoolOptions::new().connect_lazy("postgres://postgres:postgres@localhost:5432/dummy").unwrap();
        Data::new(SupplierRepo::new(pool))
    }

    fn dummy_publisher() -> Data<StreamPublisher> {
        Data::new(StreamPublisher::noop())
    }

    #[actix_web::test]
    async fn test_update_supplier_status_missing_header() {
        let req = test::TestRequest::put()
            .uri("/suppliers/00000000-0000-0000-0000-000000000000/status")
            .to_http_request();
        // create another request for respond_to
        let req_for_respond = test::TestRequest::default().to_http_request();

        let json_body = web::Json(UpdateSupplierStatusRequest {
            status: crate::models::SupplierStatus::Active,
        });

        let path = web::Path::from(uuid::Uuid::new_v4());

        let res = update_supplier_status(
            dummy_repo(),
            dummy_publisher(),
            path,
            json_body,
            req,
        ).await;

        use actix_web::Responder;
        let res = res.respond_to(&req_for_respond);
        assert_eq!(res.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn test_update_supplier_invalid_header() {
        let req = test::TestRequest::put()
            .uri("/suppliers/00000000-0000-0000-0000-000000000000")
            .insert_header(("X-User-Id", "not-a-uuid"))
            .to_http_request();
        let req_for_respond = test::TestRequest::default().to_http_request();

        let json_body = web::Json(UpdateSupplierRequest {
            legal_name: None,
            display_name: None,
            tax_id: None,
            country: None,
            platform_fee_percent: None,
            metadata: None,
        });

        let path = web::Path::from(uuid::Uuid::new_v4());

        let res = update_supplier(
            dummy_repo(),
            dummy_publisher(),
            path,
            json_body,
            req,
        ).await;

        use actix_web::Responder;
        let res = res.respond_to(&req_for_respond);
        assert_eq!(res.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }
}
