pub mod test_context;

use reqwest::Client;
use sqlx::PgPool;
use std::env;
use std::time::Duration;
use uuid::Uuid;

pub struct TestHarness {
    pub db_pool: Option<PgPool>,
    pub redis_client: Option<redis::Client>,
    pub redis_url: String,
    pub http_client: Client,
    pub gateway_url: String,
    pub jwt_secret: String,
}

impl TestHarness {
    pub async fn new() -> Self {
        let db_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/orders".to_string());
        let redis_url =
            env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        let gateway_url =
            env::var("GATEWAY_URL").unwrap_or_else(|_| "http://127.0.0.1:80".to_string());
        let jwt_secret = env::var("SECRET").unwrap_or_else(|_| "something".to_string());

        let db_pool = PgPool::connect(&db_url).await.ok();

        let redis_client = redis::Client::open(redis_url.clone()).ok();

        let http_client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        Self {
            db_pool,
            redis_client,
            redis_url,
            http_client,
            gateway_url,
            jwt_secret,
        }
    }

    pub async fn set_tenant_session(
        conn: &mut sqlx::PgConnection,
        tenant_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        let query = format!("SET LOCAL app.current_tenant_id = '{}'", tenant_id);
        sqlx::query(&query).execute(conn).await?;
        Ok(())
    }

    pub async fn seed_api_key_redis(
        &self,
        key: &str,
        record: &platform::tenant::ApiKeyRecord,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(client) = &self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let redis_key = format!("api_key:{}", key);
            let json = serde_json::to_string(record)?;
            redis::cmd("SET")
                .arg(&redis_key)
                .arg(&json)
                .query_async::<_, ()>(&mut conn)
                .await?;
        }
        Ok(())
    }

    pub async fn set_usage_counter(
        &self,
        tenant_id: Uuid,
        count: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(client) = &self.redis_client {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let year_month = chrono::Utc::now().format("%Y-%m").to_string();
            let redis_key = format!("usage:{}:{}", tenant_id, year_month);
            redis::cmd("SET")
                .arg(&redis_key)
                .arg(count)
                .query_async::<_, ()>(&mut conn)
                .await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_harness_initialization() {
        let harness = TestHarness::new().await;
        // jwt_secret comes from env::var("SECRET"), falling back to "something".
        // Assert it is non-empty regardless of which value is set.
        assert!(!harness.jwt_secret.is_empty(), "jwt_secret must not be empty");
        assert!(!harness.redis_url.is_empty(), "redis_url must not be empty");
        assert!(!harness.gateway_url.is_empty(), "gateway_url must not be empty");
    }
}
