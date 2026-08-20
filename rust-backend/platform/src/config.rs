use std::env;

#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub database_url: String,
    pub redis_url: String,
    pub service_port: u16,
    pub amqp_addr: Option<String>,
}

impl ServiceConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/db".to_string()),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            service_port: env::var("SERVICE_PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap_or(8080),
            amqp_addr: env::var("AMQP_ADDR").ok(),
        }
    }
}
