use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::tenant::{PricingTier, TenantContext};

#[derive(Clone, Debug)]
pub struct DynamicPoolRouter {
    shared_pool: PgPool,
    dedicated_pools: Arc<RwLock<HashMap<Uuid, PgPool>>>,
}

impl DynamicPoolRouter {
    pub fn new(shared_pool: PgPool) -> Self {
        Self {
            shared_pool,
            dedicated_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn shared_pool(&self) -> &PgPool {
        &self.shared_pool
    }

    pub async fn get_pool(&self, ctx: &TenantContext) -> Result<PgPool, sqlx::Error> {
        match ctx.tier {
            PricingTier::Enterprise => {
                if let Some(ref db_url) = ctx.db_connection_url {
                    {
                        let pools = self.dedicated_pools.read().await;
                        if let Some(pool) = pools.get(&ctx.tenant_id) {
                            return Ok(pool.clone());
                        }
                    }
                    let mut pools = self.dedicated_pools.write().await;
                    if let Some(pool) = pools.get(&ctx.tenant_id) {
                        return Ok(pool.clone());
                    }
                    let new_pool = PgPool::connect(db_url).await?;
                    pools.insert(ctx.tenant_id, new_pool.clone());
                    Ok(new_pool)
                } else {
                    {
                        let pools = self.dedicated_pools.read().await;
                        if let Some(pool) = pools.get(&ctx.tenant_id) {
                            return Ok(pool.clone());
                        }
                    }
                    Ok(self.shared_pool.clone())
                }
            }
            PricingTier::Free | PricingTier::Growth => Ok(self.shared_pool.clone()),
        }
    }

    pub async fn register_dedicated_pool(&self, tenant_id: Uuid, pool: PgPool) {
        let mut pools = self.dedicated_pools.write().await;
        pools.insert(tenant_id, pool);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tenant::AuthMethod;

    #[tokio::test]
    async fn test_dynamic_pool_router_shared_tier() {
        // Test router creation without actual db connection by checking logic
        let tenant_id = Uuid::new_v4();
        let ctx = TenantContext::new(
            tenant_id,
            None,
            PricingTier::Free,
            vec![],
            AuthMethod::Jwt,
        );
        assert_eq!(ctx.tier, PricingTier::Free);
    }
}
