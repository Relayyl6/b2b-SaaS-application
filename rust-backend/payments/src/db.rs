use crate::models::{CreatePaymentIntentRequest, PaymentIntent, PaymentStatus, PaymentWebhook};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct PaymentRepo {
    pool: PgPool,
}

impl PaymentRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_intent(
        &self,
        req: &CreatePaymentIntentRequest,
    ) -> Result<PaymentIntent, sqlx::Error> {
        sqlx::query_as::<_, PaymentIntent>(
            r#"
            INSERT INTO payment_intents (
                idempotency_key, order_id, user_id, supplier_id, product_id, quantity, amount, currency, provider, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, COALESCE($8, 'NGN'), COALESCE($9, 'manual'), COALESCE($10, '{}'::jsonb))
            ON CONFLICT(idempotency_key) DO UPDATE SET updated_at = payment_intents.updated_at
            RETURNING *
            "#,
        )
        .bind(&req.idempotency_key)
        .bind(req.order_id)
        .bind(req.user_id)
        .bind(req.supplier_id)
        .bind(req.product_id)
        .bind(req.quantity)
        .bind(req.amount)
        .bind(&req.currency)
        .bind(&req.provider)
        .bind(req.metadata.as_ref())
        .fetch_one(&self.pool)
        .await
    }

    pub async fn get(&self, id: Uuid) -> Result<PaymentIntent, sqlx::Error> {
        sqlx::query_as::<_, PaymentIntent>("SELECT * FROM payment_intents WHERE id = $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
    }

    pub async fn apply_webhook(
        &self,
        webhook: &PaymentWebhook,
    ) -> Result<PaymentIntent, sqlx::Error> {
        sqlx::query_as::<_, PaymentIntent>(
            r#"
            UPDATE payment_intents
            SET
                status = $1,
                provider_reference = COALESCE($2, provider_reference),
                metadata = COALESCE($3, metadata),
                updated_at = NOW()
            WHERE
                ($4::text IS NOT NULL AND idempotency_key = $4)
                OR ($2::text IS NOT NULL AND provider_reference = $2)
            RETURNING *
            "#,
        )
        .bind(&webhook.status)
        .bind(&webhook.provider_reference)
        .bind(webhook.metadata.as_ref())
        .bind(&webhook.idempotency_key)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: PaymentStatus,
    ) -> Result<PaymentIntent, sqlx::Error> {
        sqlx::query_as::<_, PaymentIntent>(
            "UPDATE payment_intents SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
        )
        .bind(status)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }
}
