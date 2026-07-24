use hex;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_json::Value;
use sha2::Sha256;
use std::env;

#[derive(Debug, Clone)]
pub struct StripeClient {
    client: Client,
    secret_key: String,
    webhook_secret: Option<String>,
}

#[derive(Debug)]
pub struct StripeIntentResponse {
    pub id: String,
    pub client_secret: String,
}

impl StripeClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            secret_key: env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
            webhook_secret: env::var("STRIPE_WEBHOOK_SECRET").ok(),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.secret_key.is_empty()
    }

    pub async fn create_payment_intent(
        &self,
        amount_cents: i64,
        currency: &str,
        metadata: Option<Value>,
        idempotency_key: &str,
    ) -> Result<StripeIntentResponse, String> {
        if !self.is_configured() {
            // Mock response if Stripe is not configured
            return Ok(StripeIntentResponse {
                id: format!("pi_mock_{}", uuid::Uuid::new_v4()),
                client_secret: "mock_client_secret".to_string(),
            });
        }

        let mut form = vec![
            ("amount", amount_cents.to_string()),
            ("currency", currency.to_string()),
        ];

        if let Some(meta) = metadata {
            if let Some(obj) = meta.as_object() {
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        form.push((format!("metadata[{k}]").leak(), s.to_string()));
                    }
                }
            }
        }

        let response = self
            .client
            .post("https://api.stripe.com/v1/payment_intents")
            .basic_auth(&self.secret_key, Some(""))
            .header("Idempotency-Key", idempotency_key)
            .form(&form)
            .send()
            .await
            .map_err(|e| format!("Stripe API error: {e}"))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Stripe returned error: {error_text}"));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| format!("Stripe JSON parse error: {e}"))?;

        let id = json["id"].as_str().unwrap_or_default().to_string();
        let client_secret = json["client_secret"].as_str().unwrap_or_default().to_string();

        Ok(StripeIntentResponse { id, client_secret })
    }

    pub async fn cancel_payment_intent(&self, stripe_id: &str) -> Result<(), String> {
        if !self.is_configured() || stripe_id.starts_with("pi_mock_") {
            return Ok(());
        }

        let response = self
            .client
            .post(&format!(
                "https://api.stripe.com/v1/payment_intents/{stripe_id}/cancel"
            ))
            .basic_auth(&self.secret_key, Some(""))
            .send()
            .await
            .map_err(|e| format!("Stripe API error: {e}"))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Stripe returned error: {error_text}"));
        }

        Ok(())
    }

    pub async fn refund_payment(&self, stripe_id: &str, amount_cents: Option<i64>, idempotency_key: Option<&str>) -> Result<(), String> {
        if !self.is_configured() || stripe_id.starts_with("pi_mock_") {
            return Ok(());
        }

        let mut form = vec![("payment_intent", stripe_id.to_string())];
        if let Some(amt) = amount_cents {
            form.push(("amount", amt.to_string()));
        }

        let mut req = self
            .client
            .post("https://api.stripe.com/v1/refunds")
            .basic_auth(&self.secret_key, Some(""))
            .form(&form);
            
        if let Some(key) = idempotency_key {
            req = req.header("Idempotency-Key", key);
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("Stripe API error: {e}"))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Stripe returned error on refund: {error_text}"));
        }

        Ok(())
    }

    pub async fn transfer_to_supplier(
        &self,
        amount_cents: i64,
        currency: &str,
        destination_account: &str,
        idempotency_key: Option<&str>,
    ) -> Result<String, String> {
        if !self.is_configured() || destination_account.starts_with("acct_mock_") {
            return Ok(format!("tr_mock_{}", uuid::Uuid::new_v4()));
        }

        let form = vec![
            ("amount", amount_cents.to_string()),
            ("currency", currency.to_string()),
            ("destination", destination_account.to_string()),
        ];

        let mut req = self
            .client
            .post("https://api.stripe.com/v1/transfers")
            .basic_auth(&self.secret_key, Some(""))
            .form(&form);
            
        if let Some(key) = idempotency_key {
            req = req.header("Idempotency-Key", key);
        }

        let response = req
            .send()
            .await
            .map_err(|e| format!("Stripe API error: {e}"))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Stripe returned error on transfer: {error_text}"));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| format!("Stripe JSON parse error: {e}"))?;

        let id = json["id"].as_str().unwrap_or_default().to_string();
        Ok(id)
    }

    pub fn verify_webhook_signature(
        &self,
        payload: &str,
        sig_header: &str,
    ) -> Result<(), &'static str> {
        let Some(webhook_secret) = &self.webhook_secret else {
            if self.is_configured() {
                return Err("Webhook secret is required when Stripe is configured");
            }
            // Accept all ONLY if Stripe is totally mocked
            return Ok(());
        };

        // Parse signature header (e.g. t=1492774577,v1=5257a869e7ecebe...)
        let mut t = None;
        let mut v1 = None;

        for pair in sig_header.split(',') {
            let mut parts = pair.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                match key {
                    "t" => t = Some(value),
                    "v1" => v1 = Some(value),
                    _ => {}
                }
            }
        }

        let (timestamp, signature) = match (t, v1) {
            (Some(t), Some(s)) => (t, s),
            _ => return Err("Invalid Stripe signature header format"),
        };

        let signed_payload = format!("{}.{}", timestamp, payload);

        let mut mac = Hmac::<Sha256>::new_from_slice(webhook_secret.as_bytes())
            .map_err(|_| "Invalid HMAC key")?;
        
        mac.update(signed_payload.as_bytes());
        let expected_sig = hex::encode(mac.finalize().into_bytes());

        if signature != expected_sig {
            return Err("Stripe signature mismatch");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    use hex;

    #[test]
    fn test_verify_webhook_signature_success() {
        let mut client = StripeClient::new();
        client.secret_key = "sk_test_123".to_string();
        client.webhook_secret = Some("whsec_test".to_string());

        let payload = r#"{"id":"evt_test"}"#;
        let timestamp = "1234567890";
        let signed_payload = format!("{}.{}", timestamp, payload);

        let mut mac = Hmac::<Sha256>::new_from_slice("whsec_test".as_bytes()).unwrap();
        mac.update(signed_payload.as_bytes());
        let expected_sig = hex::encode(mac.finalize().into_bytes());

        let sig_header = format!("t={},v1={}", timestamp, expected_sig);

        assert!(client.verify_webhook_signature(payload, &sig_header).is_ok());
    }

    #[test]
    fn test_verify_webhook_signature_failure() {
        let mut client = StripeClient::new();
        client.secret_key = "sk_test_123".to_string();
        client.webhook_secret = Some("whsec_test".to_string());

        let payload = r#"{"id":"evt_test"}"#;
        let sig_header = "t=1234567890,v1=invalid_signature";

        assert!(client.verify_webhook_signature(payload, &sig_header).is_err());
    }

    #[test]
    fn test_verify_webhook_signature_unconfigured() {
        let mut client = StripeClient::new();
        client.secret_key = "".to_string();
        client.webhook_secret = None;

        let payload = r#"{"id":"evt_test"}"#;
        let sig_header = "t=1234567890,v1=invalid_signature";

        assert!(client.verify_webhook_signature(payload, &sig_header).is_ok());
    }
}
