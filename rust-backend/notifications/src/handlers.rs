use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::NotificationRepo;
use crate::models::{
    CreateNotificationRequest, ListNotificationsQuery, Notification, NotificationChannel, NotificationDevice, RegisterDeviceRequest, UpdatePreferencesRequest, UserPreference,
};
use crate::provider::NotificationProvider;

#[utoipa::path(
    post,
    path = "/notifications",
    request_body = CreateNotificationRequest,
    responses((status = 201, description = "Notification created", body = Notification))
)]
pub async fn create_notification(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    provider: web::Data<NotificationProvider>,
    req: web::Json<CreateNotificationRequest>,
) -> impl Responder {
    let request = req.into_inner();
    let pool = match db_router.get_pool(&tenant).await {
        Ok(pool) => pool,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    if request.channel == NotificationChannel::Push && request.recipient.is_none() {
        let Some(user_id) = request.user_id else {
            return HttpResponse::BadRequest()
                .body("push notifications require either recipient push token or user_id");
        };

        let devices = match NotificationRepo::list_user_devices(&mut *tx, tenant.tenant_id, user_id).await {
            Ok(devices) => devices,
            Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
        };

        if devices.is_empty() {
            return HttpResponse::Accepted().json(serde_json::json!({
                "message": "no active push devices registered for user",
                "user_id": user_id,
                "sent": []
            }));
        }

        let mut sent = Vec::new();
        for device in devices {
            let mut device_request = request.clone();
            device_request.recipient = Some(device.push_token.clone());
            let mut payload = device_request
                .payload
                .take()
                .unwrap_or_else(|| serde_json::json!({}));
            if let Some(object) = payload.as_object_mut() {
                object.insert("device_id".to_string(), serde_json::json!(device.id));
                object.insert(
                    "push_provider".to_string(),
                    serde_json::json!(device.provider),
                );
            }
            device_request.payload = Some(payload);

            match NotificationRepo::create(&mut *tx, tenant.tenant_id, &device_request).await {
                Ok(notification) => {
                    let id = notification.id;
                    let delivered = match provider.send(&notification).await {
                        Ok(()) => NotificationRepo::mark_sent(&mut *tx, id).await,
                        Err(error) => NotificationRepo::mark_failed(&mut *tx, id, &error).await,
                    };

                    match delivered {
                        Ok(notification) => sent.push(notification),
                        Err(e) => {
                            return HttpResponse::InternalServerError()
                                .body(format!("db error: {e}"));
                        }
                    }
                }
                Err(e) => {
                    return HttpResponse::InternalServerError().body(format!("db error: {e}"))
                }
            }
        }

        if let Err(e) = tx.commit().await {
            return HttpResponse::InternalServerError().body(format!("db commit error: {e}"));
        }
        return HttpResponse::Created().json(sent);
    }

    match NotificationRepo::create(&mut *tx, tenant.tenant_id, &request).await {
        Ok(notification) => {
            let id = notification.id;
            let res = match provider.send(&notification).await {
                Ok(()) => match NotificationRepo::mark_sent(&mut *tx, id).await {
                    Ok(sent) => HttpResponse::Created().json(sent),
                    Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
                },
                Err(error) => match NotificationRepo::mark_failed(&mut *tx, id, &error).await {
                    Ok(failed) => HttpResponse::Accepted().json(failed),
                    Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
                },
            };
            if let Err(e) = tx.commit().await {
                return HttpResponse::InternalServerError().body(format!("db commit error: {e}"));
            }
            res
        }
        Err(sqlx::Error::Protocol(msg)) if msg.contains("opted out") => {
            HttpResponse::Accepted().json(serde_json::json!({
                "status": "skipped",
                "message": msg
            }))
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    get,
    path = "/notifications",
    params(ListNotificationsQuery),
    responses((status = 200, description = "Notifications listed", body = Vec<Notification>))
)]
pub async fn list_notifications(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    query: web::Query<ListNotificationsQuery>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(pool) => pool,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match NotificationRepo::list(&mut *tx, tenant.tenant_id, &query.into_inner()).await {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    get,
    path = "/notifications/{id}",
    params(("id" = Uuid, Path, description = "Notification ID")),
    responses((status = 200, description = "Notification found", body = Notification))
)]
pub async fn get_notification(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(pool) => pool,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match NotificationRepo::get(&mut *tx, tenant.tenant_id, path.into_inner()).await {
        Ok(notification) => HttpResponse::Ok().json(notification),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("notification not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    put,
    path = "/notifications/{id}/read",
    params(("id" = Uuid, Path, description = "Notification ID")),
    responses((status = 200, description = "Notification marked read", body = Notification))
)]
pub async fn mark_notification_read(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(pool) => pool,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match NotificationRepo::mark_read(&mut *tx, tenant.tenant_id, path.into_inner()).await {
        Ok(notification) => {
            let _ = tx.commit().await;
            HttpResponse::Ok().json(notification)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("notification not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    post,
    path = "/notification-devices",
    request_body = RegisterDeviceRequest,
    responses((status = 201, description = "Device registered", body = NotificationDevice))
)]
pub async fn register_device(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    req: web::Json<RegisterDeviceRequest>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(pool) => pool,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match NotificationRepo::register_device(&mut *tx, tenant.tenant_id, &req).await {
        Ok(device) => {
            let _ = tx.commit().await;
            HttpResponse::Created().json(device)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    get,
    path = "/notification-devices/user/{user_id}",
    params(("user_id" = Uuid, Path, description = "User ID")),
    responses((status = 200, description = "User devices", body = Vec<NotificationDevice>))
)]
pub async fn list_user_devices(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(pool) => pool,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match NotificationRepo::list_user_devices(&mut *tx, tenant.tenant_id, path.into_inner()).await {
        Ok(devices) => HttpResponse::Ok().json(devices),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    delete,
    path = "/notification-devices/{id}",
    params(("id" = Uuid, Path, description = "Device ID")),
    responses((status = 200, description = "Device disabled", body = NotificationDevice))
)]
pub async fn disable_device(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(pool) => pool,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match NotificationRepo::disable_device(&mut *tx, tenant.tenant_id, path.into_inner()).await {
        Ok(device) => {
            let _ = tx.commit().await;
            HttpResponse::Ok().json(device)
        }
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("device not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    get,
    path = "/notification-preferences/user/{user_id}",
    params(("user_id" = Uuid, Path, description = "User ID")),
    responses((status = 200, description = "User preferences", body = UserPreference))
)]
pub async fn get_preferences(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(pool) => pool,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match NotificationRepo::get_preferences(&mut *tx, tenant.tenant_id, path.into_inner()).await {
        Ok(prefs) => HttpResponse::Ok().json(prefs),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    put,
    path = "/notification-preferences/user/{user_id}",
    params(("user_id" = Uuid, Path, description = "User ID")),
    request_body = UpdatePreferencesRequest,
    responses((status = 200, description = "User preferences updated", body = UserPreference))
)]
pub async fn update_preferences(
    tenant: web::ReqData<platform::tenant::TenantContext>,
    db_router: web::Data<platform::db_router::DynamicPoolRouter>,
    path: web::Path<Uuid>,
    req: web::Json<UpdatePreferencesRequest>,
) -> impl Responder {
    let pool = match db_router.get_pool(&tenant).await {
        Ok(pool) => pool,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db pool error: {e}")),
    };
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(format!("db error: {e}")),
    };
    if let Err(e) = tenant.apply_rls(&mut *tx).await {
        return HttpResponse::InternalServerError().body(format!("rls error: {e}"));
    }

    match NotificationRepo::update_preferences(&mut *tx, tenant.tenant_id, path.into_inner(), &req).await {
        Ok(prefs) => {
            let _ = tx.commit().await;
            HttpResponse::Ok().json(prefs)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Health check"))
)]
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok", "service": "notifications" }))
}
