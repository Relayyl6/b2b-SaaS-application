use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use uuid::Uuid;

use crate::db::LogisticsRepo;
use crate::models::{
    CreateShipmentRequest, ListShipmentQuery, LogisticsEvent, UpdateShipmentStatusRequest,
};
use crate::publisher::RedisPublisher;

/// Create a new shipment and publish a `logistics.shipment_created` event.
///
/// On success responds with HTTP 201 Created and the created shipment as JSON; on failure responds with HTTP 500 and an error message.
///
/// # Examples
///
/// ```no_run
/// use actix_web::web;
/// use crate::models::CreateShipmentRequest;
///
/// // `repo` and `redis_pub` are `web::Data` wrappers around your implementations.
/// let repo = web::Data::new(/* LogisticsRepo instance */);
/// let redis_pub = web::Data::new(/* RedisPublisher instance */);
/// let req = web::Json(CreateShipmentRequest { /* fields */ });
///
/// // call the handler (typically awaited inside an async runtime)
/// let _resp = create_shipment(repo, redis_pub, req).await;
/// ```
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

/// Retrieve shipment details by ID.
///
/// Returns HTTP 200 with the shipment serialized as JSON when found. If no shipment exists for the given ID returns HTTP 404 with the body "shipment not found". On other database errors returns HTTP 500 with an error message.
///
/// # Examples
///
/// ```
/// use uuid::Uuid;
/// // In tests, construct a repo mock and call the handler through Actix test utilities,
/// // passing `web::Path::from(id)`; the handler responds with 200/404/500 as documented.
/// let id = Uuid::parse_str("3fa85f64-5717-4562-b3fc-2c963f66afa6").unwrap();
/// ```
pub async fn get_shipment(repo: web::Data<LogisticsRepo>, path: web::Path<Uuid>) -> impl Responder {
    match repo.get_shipment(path.into_inner()).await {
        Ok(shipment) => HttpResponse::Ok().json(shipment),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("shipment not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("db error: {e}")),
    }
}

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

/// Update a shipment's status and emit a `logistics.shipment_updated` event.
///
/// On success returns the updated shipment as JSON with HTTP 200 OK. If the requested
/// status transition is invalid the handler responds with HTTP 400 Bad Request and the
/// repository error message. If the shipment does not exist it responds with HTTP 404 Not Found.
/// For other repository failures it responds with HTTP 500 Internal Server Error.
/// Publishing the `logistics.shipment_updated` event to Redis is attempted but any publish
/// failure is logged and does not change the HTTP response.
///
/// # Examples
///
/// ```no_run
/// use actix_web::{web, http::StatusCode};
/// use uuid::Uuid;
///
/// // pseudo-code illustrating usage; replace with real repo, redis_pub and request in tests
/// # async fn example(repo: web::Data<_>, redis_pub: web::Data<_>) {
/// let shipment_id = Uuid::new_v4();
/// let req = web::Json(/* UpdateShipmentStatusRequest */);
/// let resp = update_status(repo, redis_pub, web::Path::from(shipment_id), req).await;
/// assert!(resp.respond_to(&actix_web::HttpRequest::default()).status() == StatusCode::OK
///     || resp.respond_to(&actix_web::HttpRequest::default()).status() == StatusCode::BAD_REQUEST
///     || resp.respond_to(&actix_web::HttpRequest::default()).status() == StatusCode::NOT_FOUND);
/// # }
/// ```
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
