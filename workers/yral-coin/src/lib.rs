mod coin;
mod consts;
mod error;
mod jwt;
mod types;

use candid::Principal;
use worker::*;
use worker_utils::{jwt::verify_jwt_from_header, parse_principal, RequestInitBuilder};

use crate::{
    jwt::{JWT_AUD, JWT_PUBKEY},
    types::YralBalanceUpdateRequest,
};

fn cors_policy() -> Cors {
    Cors::new()
        .with_origins(["*"])
        .with_methods([Method::Head, Method::Get, Method::Post, Method::Options])
        .with_allowed_headers(vec!["*"])
        .with_max_age(86400)
}

fn get_yral_state_stub<T>(ctx: &RouteContext<T>, user_principal: Principal) -> Result<Stub> {
    let state_ns = ctx.durable_object("USER_YRAL_COIN_STATE")?;
    let state_obj = state_ns.id_from_name(&user_principal.to_text())?;
    let state_stub = state_obj.get_stub()?;

    Ok(state_stub)
}

async fn user_yral_balance(ctx: RouteContext<()>) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");
    let game_stub = get_yral_state_stub(&ctx, user_principal)?;

    let res = game_stub
        .fetch_with_str("http://fake_url.com/balance")
        .await?;

    Ok(res)
}

async fn update_yral_balance(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    };

    let user_principal = parse_principal!(ctx, "user_principal");
    let game_stub = get_yral_state_stub(&ctx, user_principal)?;

    let req_data: YralBalanceUpdateRequest = serde_json::from_str(&req.text().await?)?;

    let req = Request::new_with_init(
        "http://fake_url.com/update_balance",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req_data)?
            .build(),
    )?;

    game_stub.fetch_with_request(req).await
}

async fn estabilish_balance_ws(ctx: RouteContext<()>) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");
    let game_stub = get_yral_state_stub(&ctx, user_principal)?;

    let headers = Headers::new();
    headers.set("Upgrade", "websocket")?;
    let new_req = Request::new_with_init(
        "http://fake_url.com/ws/balance",
        RequestInitBuilder::default()
            .method(Method::Get)
            .replace_headers(headers)
            .build(),
    )?;

    game_stub.fetch_with_request(new_req).await
}

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    let router = Router::new();

    let res = router
        .get_async("/balance/:user_principal", |_req, ctx| {
            user_yral_balance(ctx)
        })
        .post_async("/update_balance/:user_principal", update_yral_balance)
        .get_async("/ws/balance/:user_principal", |_req, ctx| {
            estabilish_balance_ws(ctx)
        })
        .options("/*catchall", |_, _| Response::empty())
        .run(req, env)
        .await?;

    res.with_cors(&cors_policy())
}
