use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricingTier {
    Free,
    Growth,
    Enterprise,
}

impl Default for PricingTier {
    fn default() -> Self {
        PricingTier::Free
    }
}

impl PricingTier {
    pub fn monthly_limit(&self) -> u64 {
        match self {
            PricingTier::Free => 100,
            PricingTier::Growth => 10_000,
            PricingTier::Enterprise => u64::MAX,
        }
    }
}

impl fmt::Display for PricingTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PricingTier::Free => write!(f, "Free"),
            PricingTier::Growth => write!(f, "Growth"),
            PricingTier::Enterprise => write!(f, "Enterprise"),
        }
    }
}

impl FromStr for PricingTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "free" => Ok(PricingTier::Free),
            "growth" => Ok(PricingTier::Growth),
            "enterprise" => Ok(PricingTier::Enterprise),
            _ => Err(format!("Unknown pricing tier: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    Jwt,
    ApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub tier: PricingTier,
    pub permissions: Vec<String>,
    pub feature_flags: HashMap<String, bool>,
    pub auth_method: AuthMethod,
    pub db_connection_url: Option<String>,
}

impl TenantContext {
    pub fn new(
        tenant_id: Uuid,
        user_id: Option<Uuid>,
        tier: PricingTier,
        permissions: Vec<String>,
        auth_method: AuthMethod,
    ) -> Self {
        Self {
            tenant_id,
            user_id,
            tier,
            permissions,
            feature_flags: HashMap::new(),
            auth_method,
            db_connection_url: None,
        }
    }

    pub fn with_db_connection_url(mut self, url: impl Into<String>) -> Self {
        self.db_connection_url = Some(url.into());
        self
    }

    pub async fn apply_rls<'c, E>(&self, executor: E) -> Result<(), sqlx::Error>
    where
        E: sqlx::Executor<'c, Database = sqlx::Postgres>,
    {
        let query = format!("SET LOCAL app.current_tenant_id = '{}';", self.tenant_id);
        sqlx::query(&query).execute(executor).await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub key_prefix: String,
    pub key_hash: String,
    pub permissions: Vec<String>,
    pub rate_limit_override: Option<u64>,
    pub is_active: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pricing_tier_limits() {
        assert_eq!(PricingTier::Free.monthly_limit(), 100);
        assert_eq!(PricingTier::Growth.monthly_limit(), 10_000);
        assert_eq!(PricingTier::Enterprise.monthly_limit(), u64::MAX);
    }

    #[test]
    fn test_pricing_tier_from_str() {
        assert_eq!("Free".parse::<PricingTier>().unwrap(), PricingTier::Free);
        assert_eq!("growth".parse::<PricingTier>().unwrap(), PricingTier::Growth);
        assert_eq!("ENTERPRISE".parse::<PricingTier>().unwrap(), PricingTier::Enterprise);
        assert!("invalid".parse::<PricingTier>().is_err());
    }

    #[test]
    fn test_tenant_context_creation() {
        let tenant_id = Uuid::new_v4();
        let ctx = TenantContext::new(
            tenant_id,
            None,
            PricingTier::Growth,
            vec!["orders:read".to_string()],
            AuthMethod::ApiKey,
        );
        assert_eq!(ctx.tenant_id, tenant_id);
        assert_eq!(ctx.tier, PricingTier::Growth);
        assert_eq!(ctx.auth_method, AuthMethod::ApiKey);
    }

    #[test]
    fn test_tenant_context_apply_rls_sql() {
        let tenant_id = Uuid::new_v4();
        let ctx = TenantContext::new(
            tenant_id,
            None,
            PricingTier::Growth,
            vec!["orders:read".to_string()],
            AuthMethod::ApiKey,
        );
        let query = format!("SET LOCAL app.current_tenant_id = '{}';", ctx.tenant_id);
        assert!(query.contains(&tenant_id.to_string()));
        assert!(query.contains("SET LOCAL app.current_tenant_id"));
    }
}
