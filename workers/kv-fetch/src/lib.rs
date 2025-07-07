use std::collections::HashSet;

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::json;
use worker::*;

mod utils;

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .post_async("/:key", |mut req, ctx| async move {
            if let Err((msg, status)) = verify_jwt_token(&req, &ctx) {
                return Response::error(msg, status);
            }

            let Some(key) = ctx.param("key") else {
                return Response::error("expected key and value path params", 400);
            };

            let value = match req.text().await {
                Ok(value) => value,
                Err(err) => return Response::error(format!("unable to parse value: {err}"), 400),
            };

            if value.is_empty() {
                return Response::error("value cannot be empty", 400);
            }

            match ctx.kv("kvfetch")?.put(key, &value)?.execute().await {
                Ok(_) => Response::ok(
                    json! ({
                        "key": key,
                        "value": value
                    })
                    .to_string(),
                ),
                Err(err) => Response::error(format!("unable to set value: {err}"), 500),
            }
        })
        .get_async("/:key", |req, ctx| async move {
            if let Err((msg, status)) = verify_jwt_token(&req, &ctx) {
                return Response::error(msg, status);
            }

            let Some(key) = ctx.param("key") else {
                return Response::error("expected key path param", 400);
            };

            match ctx.kv("kvfetch")?.get(key).text().await? {
                Some(value) => Response::ok(value),
                None => Response::error("key not found", 404),
            }
        })
        .run(req, env)
        .await
}

#[derive(Debug, Deserialize, PartialEq)]
struct TokenClaims {
    pub sub: String,
    pub company: String,
}

fn verify_jwt_token(
    req: &Request,
    ctx: &RouteContext<()>,
) -> std::result::Result<(), (String, u16)> {
    let public_key = match ctx.secret("PUBLIC_KEY") {
        Err(err) => {
            return Err((format!("unable to fetch public key: {err}"), 500));
        }
        Ok(auth_key) => auth_key.to_string(),
    };

    let public_key = format!("-----BEGIN PUBLIC KEY-----\n{public_key}\n-----END PUBLIC KEY-----",);

    let public_key = match DecodingKey::from_ed_pem(public_key.as_bytes()) {
        Err(err) => return Err((format!("unable to parse public key: {err}"), 500)),
        Ok(key) => key,
    };

    let mut validation = Validation::new(Algorithm::EdDSA);
    validation.required_spec_claims = HashSet::new();
    validation.validate_exp = false;

    let req_token = match req.headers().get("Authorization") {
        Err(err) => return Err((format!("invalid header: {err}"), 401)),
        Ok(None) => return Err(("unauthorized".into(), 401)),
        Ok(Some(key)) => key.replace("Bearer ", ""),
    };

    let req_token = match decode::<TokenClaims>(&req_token, &public_key, &validation) {
        Err(err) => return Err((format!("failed to decode token: {err}"), 401)),
        Ok(req_token) => req_token,
    };

    let claims = req_token.claims;

    if claims.sub != "hot-or-not-web-leptos-ssr" || claims.company != "gobazzinga" {
        return Err(("unauthorized".into(), 401));
    }

    Ok(())
}
