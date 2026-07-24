use actix_web::{web, HttpResponse, Responder};
use uuid::Uuid;

use crate::db::NotificationRepo;
use crate::models::{
    CreateNotificationRequest, ListNotificationsQuery, NotificationChannel, RegisterDeviceRequest, UpdatePreferencesRequest,
};
use crate::provider::NotificationProvider;

pub async fn create_notification(
    repo: web::Data<NotificationRepo>,
    provider: web::Data<NotificationProvider>,
    req: web::Json<CreateNotificationRequest>,
) -> impl Responder {
    let request = req.into_inner();

    if request.channel == NotificationChannel::Push && request.recipient.is_none() {
        let Some(user_id) = request.user_id else {
            return HttpResponse::BadRequest()
                .body("push notifications require either recipient push token or user_id");
        };

        let devices = match repo.list_user_devices(user_id).await {
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

            match repo.create(&device_request).await {
                Ok(notification) => {
                    let id = notification.id;
                    let delivered = match provider.send(&notification).await {
                        Ok(()) => repo.mark_sent(id).await,
                        Err(error) => repo.mark_failed(id, &error).await,
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

        return HttpResponse::Created().json(sent);
    }

    match repo.create(&request).await {
        Ok(notification) => {
            let id = notification.id;
            match provider.send(&notification).await {
                Ok(()) => match repo.mark_sent(id).await {
                    Ok(sent) => HttpResponse::Created().json(sent),
                    Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
                },
                Err(error) => match repo.mark_failed(id, &error).await {
                    Ok(failed) => HttpResponse::Accepted().json(failed),
                    Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
                },
            }
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

pub async fn list_notifications(
    repo: web::Data<NotificationRepo>,
    query: web::Query<ListNotificationsQuery>,
) -> impl Responder {
    match repo.list(&query.into_inner()).await {
        Ok(notifications) => HttpResponse::Ok().json(notifications),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn get_notification(
    repo: web::Data<NotificationRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    match repo.get(path.into_inner()).await {
        Ok(notification) => HttpResponse::Ok().json(notification),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("notification not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn mark_notification_read(
    repo: web::Data<NotificationRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    match repo.mark_read(path.into_inner()).await {
        Ok(notification) => HttpResponse::Ok().json(notification),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("notification not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn register_device(
    repo: web::Data<NotificationRepo>,
    req: web::Json<RegisterDeviceRequest>,
) -> impl Responder {
    match repo.register_device(&req).await {
        Ok(device) => HttpResponse::Created().json(device),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn list_user_devices(
    repo: web::Data<NotificationRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    match repo.list_user_devices(path.into_inner()).await {
        Ok(devices) => HttpResponse::Ok().json(devices),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn disable_device(
    repo: web::Data<NotificationRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    match repo.disable_device(path.into_inner()).await {
        Ok(device) => HttpResponse::Ok().json(device),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("device not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn get_preferences(
    repo: web::Data<NotificationRepo>,
    path: web::Path<Uuid>,
) -> impl Responder {
    match repo.get_preferences(path.into_inner()).await {
        Ok(prefs) => HttpResponse::Ok().json(prefs),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn update_preferences(
    repo: web::Data<NotificationRepo>,
    path: web::Path<Uuid>,
    req: web::Json<UpdatePreferencesRequest>,
) -> impl Responder {
    match repo.update_preferences(path.into_inner(), &req).await {
        Ok(prefs) => HttpResponse::Ok().json(prefs),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok", "service": "notifications" }))
}
