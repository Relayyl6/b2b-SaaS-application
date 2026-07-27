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

    pub async fn get_intent_by_order_id(&self, order_id: Uuid) -> Result<PaymentIntent, sqlx::Error> {
        sqlx::query_as::<_, PaymentIntent>("SELECT * FROM payment_intents WHERE order_id = $1 ORDER BY created_at DESC LIMIT 1")
            .bind(order_id)
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

    pub async fn update_provider_reference(
        &self,
        id: Uuid,
        provider_reference: &str,
        metadata: &serde_json::Value,
    ) -> Result<PaymentIntent, sqlx::Error> {
        sqlx::query_as::<_, PaymentIntent>(
            "UPDATE payment_intents SET provider_reference = $1, metadata = $2, updated_at = NOW() WHERE id = $3 RETURNING *",
        )
        .bind(provider_reference)
        .bind(metadata)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn cancel_by_order_id(&self, order_id: Uuid) -> Result<(), sqlx::Error> {
        self.cancel_by_order_id_returning(order_id).await?;
        Ok(())
    }

    pub async fn cancel_by_order_id_returning(&self, order_id: Uuid) -> Result<PaymentIntent, sqlx::Error> {
        sqlx::query_as::<_, PaymentIntent>(
            "UPDATE payment_intents SET status = 'cancelled', updated_at = NOW() WHERE order_id = $1 AND status != 'succeeded' RETURNING *"
        )
        .bind(order_id)
        .fetch_one(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    #[ignore]
    async fn test_create_and_idempotency(pool: PgPool) {
        let repo = PaymentRepo::new(pool);
        let req = CreatePaymentIntentRequest {
            idempotency_key: "test_idemp_key".to_string(),
            order_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            supplier_id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            quantity: 2,
            amount: 5000,
            currency: Some("usd".to_string()),
            provider: Some("stripe".to_string()),
            metadata: None,
        };

        // First creation
        let intent1 = repo.create_intent(&req).await.expect("Failed to create intent");
        assert_eq!(intent1.idempotency_key, "test_idemp_key");
        assert_eq!(intent1.amount, 5000);

        // Idempotent creation (same key)
        let intent2 = repo.create_intent(&req).await.expect("Failed idempotent creation");
        assert_eq!(intent1.id, intent2.id, "Idempotent request should return the same intent ID");
    }

    #[sqlx::test]
    #[ignore]
    async fn test_apply_webhook(pool: PgPool) {
        let repo = PaymentRepo::new(pool);
        let req = CreatePaymentIntentRequest {
            idempotency_key: "webhook_test_key".to_string(),
            order_id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            supplier_id: Uuid::new_v4(),
            product_id: Uuid::new_v4(),
            quantity: 1,
            amount: 1000,
            currency: None,
            provider: None,
            metadata: None,
        };

        let intent = repo.create_intent(&req).await.unwrap();

        let webhook = PaymentWebhook {
            provider_reference: None,
            idempotency_key: Some("webhook_test_key".to_string()),
            status: PaymentStatus::Succeeded,
            metadata: None,
        };

        let updated = repo.apply_webhook(&webhook).await.expect("Failed to apply webhook");
        assert_eq!(updated.id, intent.id);
        assert!(matches!(updated.status, PaymentStatus::Succeeded));
    }
}
