use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal server error")]
    Internal,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type) = match self {
            AppError::Database(_) | AppError::Internal => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
            AppError::NotFound(_) => (actix_web::http::StatusCode::NOT_FOUND, "not_found"),
            AppError::Unauthorized(_) => (actix_web::http::StatusCode::UNAUTHORIZED, "unauthorized"),
            AppError::Forbidden(_) => (actix_web::http::StatusCode::FORBIDDEN, "forbidden"),
            AppError::BadRequest(_) => (actix_web::http::StatusCode::BAD_REQUEST, "bad_request"),
        };

        HttpResponse::build(status).json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
        })
    }
}
