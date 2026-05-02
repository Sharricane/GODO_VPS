use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use rand_core::OsRng;
use uuid::Uuid;
use x25519_dalek::{PublicKey, StaticSecret};

pub struct RealityKeypair {
    pub private_key: String,
    pub public_key: String,
    pub short_id: String,
}

/// Generate X25519 keypair for VLESS-Reality (base64url, no padding).
pub fn generate_reality_keypair() -> RealityKeypair {
    let secret = StaticSecret::random_from_rng(OsRng);
    let public = PublicKey::from(&secret);

    let mut short_id_bytes = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut short_id_bytes);

    RealityKeypair {
        private_key: URL_SAFE_NO_PAD.encode(secret.as_bytes()),
        public_key: URL_SAFE_NO_PAD.encode(public.as_bytes()),
        short_id: hex::encode(short_id_bytes),
    }
}

pub fn generate_short_id() -> String {
    let mut bytes = [0u8; 8];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_hy2_password() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Parse or generate — returns (value, was_generated).
pub fn or_generate(val: Option<String>, gen: impl Fn() -> String) -> (String, bool) {
    match val {
        Some(v) if !v.is_empty() => (v, false),
        _ => (gen(), true),
    }
}

/// Derive public key from a base64url-encoded Reality private key.
pub fn public_from_private(private_b64: &str) -> Result<String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(private_b64)
        .map_err(|e| anyhow::anyhow!("bad private key base64: {e}"))?;
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("private key must be 32 bytes"))?;
    let secret = StaticSecret::from(arr);
    let public = PublicKey::from(&secret);
    Ok(URL_SAFE_NO_PAD.encode(public.as_bytes()))
}
