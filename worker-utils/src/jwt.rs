use std::collections::HashSet;

use jsonwebtoken::DecodingKey;
use serde::{Deserialize, Serialize};
use worker::Request;

use crate::environment::{RunEnv, env_kind};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: String,
    pub exp: usize,
}

pub fn verify_jwt(
    public_key_pem: &str,
    aud: String,
    jwt: &str,
) -> Result<(), jsonwebtoken::errors::Error> {
    let mut validation = jsonwebtoken::Validation::default();
    validation.aud = Some(HashSet::from([aud]));
    validation.algorithms = vec![jsonwebtoken::Algorithm::EdDSA];
    validation.validate_exp = false;

    jsonwebtoken::decode::<Claims>(
        jwt,
        &DecodingKey::from_ed_pem(public_key_pem.as_bytes()).unwrap(),
        &validation,
    )?;

    Ok(())
}

pub fn verify_jwt_from_header(
    public_key_pem: &str,
    aud: String,
    req: &Request,
) -> Result<(), (String, u16)> {
    if env_kind() == RunEnv::Mock || env_kind() == RunEnv::Local {
        println!("Skipping JWT verification in mock/local environment");
        return Ok(());
    }

    let jwt = req
        .headers()
        .get("Authorization")
        .ok()
        .flatten()
        .ok_or_else(|| ("missing Authorization header".to_string(), 401))?;

    let jwt = jwt.to_string();
    if !jwt.starts_with("Bearer ") {
        return Err(("invalid Authorization header".to_string(), 401));
    }

    let jwt = &jwt[7..];
    verify_jwt(public_key_pem, aud, jwt).map_err(|_| ("invalid JWT".to_string(), 401))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use std::time::{SystemTime, UNIX_EPOCH};

    // These are for test only. In production, use secure keys.
    // Generated using a modern OpenSSL or age-keygen, valid Ed25519 PEM format
    const TEST_ED25519_PRIVATE_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
            MC4CAQAwBQYDK2VwBCIEIFxEb2I7tPuKvihV4PgA55HDyMoVPHs2p0/nqJOBeuGG
        -----END PRIVATE KEY-----";
    const TEST_ED25519_PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----
        MCowBQYDK2VwAyEAwmK6SSAu2E9V7uynkCKEaj5nZJyTvNG4x0KohsRzLpg=
    -----END PUBLIC KEY-----";

    #[test]
    fn test_verify_jwt_no_expiry_check() {
        let aud = "test-audience".to_string();
        let claims = Claims {
            aud: aud.clone(),
            exp: 1, // Already expired, but should pass since validate_exp = false
        };
        let token = encode(
            &Header::new(Algorithm::EdDSA),
            &claims,
            &EncodingKey::from_ed_pem(TEST_ED25519_PRIVATE_KEY_PEM.as_bytes()).unwrap(),
        )
        .unwrap();
        let result = verify_jwt(TEST_ED25519_PUBLIC_KEY_PEM, aud, &token);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_jwt_with_valid_expiry() {
        let aud = "test-audience".to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        let claims = Claims {
            aud: aud.clone(),
            exp: now + 3600, // 1 hour in the future
        };
        let token = encode(
            &Header::new(Algorithm::EdDSA),
            &claims,
            &EncodingKey::from_ed_pem(TEST_ED25519_PRIVATE_KEY_PEM.as_bytes()).unwrap(),
        )
        .unwrap();
        let result = verify_jwt(TEST_ED25519_PUBLIC_KEY_PEM, aud, &token);
        assert!(result.is_ok());
    }
}
