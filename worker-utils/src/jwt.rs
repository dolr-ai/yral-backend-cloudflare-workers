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
