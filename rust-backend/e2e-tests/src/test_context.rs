use hmac::{Hmac, Mac};
use jsonwebtoken::{EncodingKey, Header, encode};
use platform::tenant::{AuthMethod, PricingTier, TenantContext};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockClaims {
    pub sub: Uuid,
    pub exp: usize,
    #[serde(default)]
    pub tenant_id: Uuid,
    #[serde(default)]
    pub tier: PricingTier,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnrichedEventPayload<T> {
    pub event_id: Uuid,
    pub event_type: String,
    pub tenant_id: Option<Uuid>,
    pub timestamp: i64,
    pub payload: T,
    pub signature: Option<String>,
}

pub struct MockTenantFixture {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub tier: PricingTier,
    pub api_key: String,
    pub jwt_token: String,
}

impl MockTenantFixture {
    pub fn new(tier: PricingTier, secret: &str) -> Self {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let api_key = generate_mock_api_key(tenant_id, "live");
        let jwt_token = generate_mock_jwt(user_id, tenant_id, tier, secret)
            .expect("Failed to generate mock JWT");

        Self {
            tenant_id,
            user_id,
            tier,
            api_key,
            jwt_token,
        }
    }

    pub fn to_tenant_context(&self) -> TenantContext {
        TenantContext::new(
            self.tenant_id,
            Some(self.user_id),
            self.tier,
            vec!["*".to_string()],
            AuthMethod::ApiKey,
        )
    }
}

pub fn generate_mock_jwt(
    user_id: Uuid,
    tenant_id: Uuid,
    tier: PricingTier,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(1))
        .unwrap()
        .timestamp() as usize;

    let claims = MockClaims {
        sub: user_id,
        exp,
        tenant_id,
        tier,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn generate_expired_jwt(
    user_id: Uuid,
    tenant_id: Uuid,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let exp = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::hours(1))
        .unwrap()
        .timestamp() as usize;

    let claims = MockClaims {
        sub: user_id,
        exp,
        tenant_id,
        tier: PricingTier::Free,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn generate_mock_api_key(tenant_id: Uuid, prefix: &str) -> String {
    format!("sk_{}_{}", prefix, tenant_id.simple())
}

pub fn compute_hmac_signature(data: &str, secret: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub fn create_enriched_event<T: Serialize>(
    event_type: &str,
    tenant_id: Option<Uuid>,
    payload: T,
    secret: &str,
) -> EnrichedEventPayload<T> {
    let event_id = Uuid::new_v4();
    let timestamp = chrono::Utc::now().timestamp();

    let sig_input = format!(
        "{}:{}:{}",
        event_id,
        tenant_id.map(|t| t.to_string()).unwrap_or_default(),
        timestamp
    );
    let signature = compute_hmac_signature(&sig_input, secret);

    EnrichedEventPayload {
        event_id,
        event_type: event_type.to_string(),
        tenant_id,
        timestamp,
        payload,
        signature: Some(signature),
    }
}

pub fn validate_event_tenant_enrichment<T>(
    event: &EnrichedEventPayload<T>,
    expected_tenant_id: Uuid,
) -> bool {
    match event.tenant_id {
        Some(tid) => tid == expected_tenant_id,
        None => false,
    }
}

pub fn format_set_tenant_session_sql(tenant_id: Uuid) -> String {
    format!("SET LOCAL app.current_tenant_id = '{}';", tenant_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_tenant_fixture_and_jwt() {
        let secret = "test_jwt_secret";
        let fixture = MockTenantFixture::new(PricingTier::Growth, secret);
        assert!(!fixture.api_key.is_empty());
        assert!(!fixture.jwt_token.is_empty());

        let event = create_enriched_event(
            "order.created",
            Some(fixture.tenant_id),
            serde_json::json!({"order_id": Uuid::new_v4()}),
            secret,
        );
        assert!(validate_event_tenant_enrichment(&event, fixture.tenant_id));
        assert!(event.signature.is_some());
    }

    #[test]
    fn test_expired_jwt_generation() {
        let secret = "test_jwt_secret";
        let user_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let token = generate_expired_jwt(user_id, tenant_id, secret).unwrap();
        assert!(!token.is_empty());
    }
}
