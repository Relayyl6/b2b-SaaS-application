use crate::models::{CreateSupplierRequest, Supplier, SupplierStatus};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct SupplierRepo {
    pool: PgPool,
}

impl SupplierRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, req: &CreateSupplierRequest) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            r#"
            INSERT INTO suppliers (owner_user_id, legal_name, display_name, tax_id, country, metadata)
            VALUES ($1, $2, $3, $4, COALESCE($5, 'NG'), COALESCE($6, '{}'::jsonb))
            RETURNING *
            "#,
        )
        .bind(req.owner_user_id)
        .bind(&req.legal_name)
        .bind(&req.display_name)
        .bind(&req.tax_id)
        .bind(&req.country)
        .bind(req.metadata.as_ref())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get(&self, id: Uuid) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as::<_, Supplier>("SELECT * FROM suppliers WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn list_by_owner(&self, owner_user_id: Uuid) -> Result<Vec<Supplier>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "SELECT * FROM suppliers WHERE owner_user_id = $1 ORDER BY created_at DESC",
        )
        .bind(owner_user_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: SupplierStatus,
    ) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "UPDATE suppliers SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(status)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }
}
