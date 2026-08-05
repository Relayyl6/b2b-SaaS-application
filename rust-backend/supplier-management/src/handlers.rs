use actix_web::{web, HttpResponse, Responder, HttpRequest};
use actix_web::web::ReqData;
use chrono::Utc;
use platform::streams::StreamPublisher;
use platform::tenant::TenantContext;
use platform::db_router::DynamicPoolRouter;
use uuid::Uuid;

use crate::db::SupplierRepo;
use crate::models::{CreateSupplierRequest, Supplier, SupplierEvent, UpdateSupplierStatusRequest, UpdateSupplierRequest};

#[utoipa::path(
    post,
    path = "/suppliers",
    request_body = CreateSupplierRequest,
    responses(
        (status = 201, description = "Supplier created successfully", body = Supplier)
    )
)]
pub async fn create_supplier(
    tenant: ReqData<TenantContext>,
    db_router: web::Data<DynamicPoolRouter>,
    repo: web::Data<SupplierRepo>,
    publisher: web::Data<StreamPublisher>,
    req: web::Json<CreateSupplierRequest>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    match repo.create(&mut tx, &req).await {
        Ok(supplier) => {
            tx.commit().await.unwrap();
            publish_supplier_event(tenant.tenant_id, &publisher, "supplier.created", &supplier);
            HttpResponse::Created().json(supplier)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    get,
    path = "/suppliers/{id}",
    params(
        ("id" = Uuid, Path, description = "Supplier ID")
    ),
    responses(
        (status = 200, description = "Supplier found", body = Supplier)
    )
)]
pub async fn get_supplier(
    tenant: ReqData<TenantContext>,
    db_router: web::Data<DynamicPoolRouter>,
    repo: web::Data<SupplierRepo>, 
    path: web::Path<Uuid>
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    match repo.get(&mut tx, path.into_inner()).await {
        Ok(supplier) => {
            tx.commit().await.unwrap();
            HttpResponse::Ok().json(supplier)
        },
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("supplier not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    get,
    path = "/suppliers/owner/{owner_user_id}",
    params(
        ("owner_user_id" = Uuid, Path, description = "Owner User ID")
    ),
    responses(
        (status = 200, description = "Owner suppliers", body = Vec<Supplier>)
    )
)]
pub async fn list_owner_suppliers(
    tenant: ReqData<TenantContext>,
    db_router: web::Data<DynamicPoolRouter>,
    repo: web::Data<SupplierRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    match repo.list_by_owner(&mut tx, path.into_inner()).await {
        Ok(suppliers) => {
            tx.commit().await.unwrap();
            HttpResponse::Ok().json(suppliers)
        },
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    put,
    path = "/suppliers/{id}/status",
    params(
        ("id" = Uuid, Path, description = "Supplier ID")
    ),
    request_body = UpdateSupplierStatusRequest,
    responses(
        (status = 200, description = "Supplier status updated", body = Supplier)
    )
)]
pub async fn update_supplier_status(
    tenant: ReqData<TenantContext>,
    db_router: web::Data<DynamicPoolRouter>,
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

    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    match repo
        .update_status(&mut tx, path.into_inner(), owner_user_id, req.status.clone())
        .await
    {
        Ok(supplier) => {
            tx.commit().await.unwrap();
            publish_supplier_event(tenant.tenant_id, &publisher, "supplier.status_updated", &supplier);
            HttpResponse::Ok().json(supplier)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("supplier not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

fn publish_supplier_event(tenant_id: Uuid, publisher: &StreamPublisher, event_type: &str, supplier: &Supplier) {
    publisher.publish_async(
        event_type,
        SupplierEvent {
            tenant_id,
            event_type: event_type.to_string(),
            supplier_id: supplier.id,
            user_id: supplier.owner_user_id,
            owner_user_id: supplier.owner_user_id,
            status: supplier.status.clone(),
            timestamp: Utc::now(),
        },
    );
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check")
    )
)]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status":"ok","service":"supplier-management"}))
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Metrics endpoint")
    )
)]
pub async fn metrics_doc() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[utoipa::path(
    put,
    path = "/suppliers/{id}",
    params(
        ("id" = Uuid, Path, description = "Supplier ID")
    ),
    request_body = UpdateSupplierRequest,
    responses(
        (status = 200, description = "Supplier updated", body = Supplier)
    )
)]
pub async fn update_supplier(
    tenant: ReqData<TenantContext>,
    db_router: web::Data<DynamicPoolRouter>,
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

    let pool = match db_router.get_pool(&tenant).await {
        Ok(p) => p,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    match repo
        .update_supplier(&mut tx, path.into_inner(), owner_user_id, &req)
        .await
    {
        Ok(supplier) => {
            tx.commit().await.unwrap();
            publish_supplier_event(tenant.tenant_id, &publisher, "supplier.updated", &supplier);
            HttpResponse::Ok().json(supplier)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("supplier not found or not owned by user"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}


