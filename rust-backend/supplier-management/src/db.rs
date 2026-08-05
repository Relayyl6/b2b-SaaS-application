use crate::models::{CreateSupplierRequest, Supplier, SupplierStatus, UpdateSupplierRequest};
use uuid::Uuid;

#[derive(Clone)]
pub struct SupplierRepo {}

impl SupplierRepo {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn create(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, req: &CreateSupplierRequest) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            r#"
            INSERT INTO suppliers (owner_user_id, legal_name, display_name, tax_id, country, metadata, platform_fee_percent)
            VALUES ($1, $2, $3, $4, COALESCE($5, 'NG'), COALESCE($6, '{}'::jsonb), $7)
            RETURNING *
            "#,
        )
        .bind(req.owner_user_id)
        .bind(&req.legal_name)
        .bind(&req.display_name)
        .bind(&req.tax_id)
        .bind(&req.country)
        .bind(req.metadata.as_ref())
        .bind(req.platform_fee_percent.unwrap_or(5.0))
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn get(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, id: Uuid) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as::<_, Supplier>("SELECT * FROM suppliers WHERE id = $1")
            .bind(id)
            .fetch_one(&mut **tx)
            .await
    }

    pub async fn list_by_owner(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, owner_user_id: Uuid) -> Result<Vec<Supplier>, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "SELECT * FROM suppliers WHERE owner_user_id = $1 ORDER BY created_at DESC",
        )
        .bind(owner_user_id)
        .fetch_all(&mut **tx)
        .await
    }

    pub async fn update_status(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
        owner_user_id: Uuid,
        status: SupplierStatus,
    ) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            "UPDATE suppliers SET status = $1, updated_at = NOW() WHERE id = $2 AND owner_user_id = $3 RETURNING *",
        )
        .bind(status)
        .bind(id)
        .bind(owner_user_id)
        .fetch_one(&mut **tx)
        .await
    }

    pub async fn update_supplier(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: Uuid,
        owner_user_id: Uuid,
        req: &UpdateSupplierRequest,
    ) -> Result<Supplier, sqlx::Error> {
        sqlx::query_as::<_, Supplier>(
            r#"
            UPDATE suppliers 
            SET 
                legal_name = COALESCE($1, legal_name),
                display_name = COALESCE($2, display_name),
                tax_id = COALESCE($3, tax_id),
                country = COALESCE($4, country),
                platform_fee_percent = COALESCE($5, platform_fee_percent),
                metadata = COALESCE($6, metadata),
                updated_at = NOW()
            WHERE id = $7 AND owner_user_id = $8
            RETURNING *
            "#,
        )
        .bind(req.legal_name.as_ref())
        .bind(req.display_name.as_ref())
        .bind(req.tax_id.as_ref())
        .bind(req.country.as_ref())
        .bind(req.platform_fee_percent)
        .bind(req.metadata.as_ref())
        .bind(id)
        .bind(owner_user_id)
        .fetch_one(&mut **tx)
        .await
    }
}
