use rand::Rng;
use sha2::{Sha256, Digest};
use bs58;

pub struct ApiKey {
    /// The full plaintext key — ONLY shown ONCE to the developer, never stored
    pub plaintext: String,
    /// The prefix (first 10 chars) — stored in DB for identification
    pub prefix: String,
    /// SHA-256 hash of the full key — stored in DB for validation
    pub hash: String,
}

pub fn generate_api_key(key_type: &str, environment: &str) -> ApiKey {
    // Generate 32 cryptographically random bytes
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen::<u8>()).collect();

    // Encode as base58 to avoid ambiguous characters (no 0, O, l, I)
    let random_part = bs58::encode(&random_bytes).into_string();

    // Assemble the full key
    let plaintext = format!("{}_{}_{}",
        key_type,       // "sk", "pk", "rk"
        environment,    // "live", "test"
        &random_part[..26]
    );

    // Compute the hash (this is what gets stored in the DB)
    let mut hasher = Sha256::new();
    hasher.update(plaintext.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    // The prefix is used to look up which hash to compare against
    let prefix = plaintext[..10].to_string(); // e.g. "sk_live_7c"

    ApiKey { plaintext, prefix, hash }
}
