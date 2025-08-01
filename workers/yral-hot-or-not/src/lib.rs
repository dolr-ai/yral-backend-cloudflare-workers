mod admin_cans;
mod backend_impl;
mod consts;
mod hon_game;
mod jwt;
mod migrate;
mod notification;
mod referral;
mod treasury;
mod utils;

use backend_impl::{StateBackend, UserStateBackendImpl};
use candid::Principal;
use hon_worker_common::{
    hon_game_vote_msg, hon_game_vote_msg_v3, hon_game_withdraw_msg, hon_referral_msg,
    AirdropClaimError, GameInfoReq, GameInfoReqV3, HoNGameVoteReq, HoNGameVoteReqV3,
    HoNGameWithdrawReq, PaginatedGamesReq, PaginatedReferralsReq, ReferralReqWithSignature,
    SatsBalanceUpdateRequest, SatsBalanceUpdateRequestV2, VerifiableClaimRequest,
    VoteRequestWithSentiment, VoteRequestWithSentimentV3, WorkerError,
};
use jwt::{JWT_AUD, JWT_PUBKEY};
use notification::{NotificationClient, NotificationType};
use serde_json::json;
use std::result::Result as StdResult;
use utils::err_to_resp;
use worker::*;
use worker_utils::{jwt::verify_jwt_from_header, parse_principal, RequestInitBuilder};

fn cors_policy() -> Cors {
    Cors::new()
        .with_origins(["*"])
        .with_methods([Method::Head, Method::Get, Method::Post, Method::Options])
        .with_allowed_headers(vec!["*"])
        .with_max_age(86400)
}

fn verify_hon_game_req(
    sender: Principal,
    req: &HoNGameVoteReq,
) -> StdResult<(), (u16, WorkerError)> {
    let msg = hon_game_vote_msg(req.request.clone());

    req.signature
        .clone()
        .verify_identity(sender, msg)
        .map_err(|_| (401, WorkerError::InvalidSignature))?;

    Ok(())
}

fn verify_hon_game_req_v3(
    sender: Principal,
    req: &HoNGameVoteReqV3,
) -> StdResult<(), (u16, WorkerError)> {
    let msg = hon_game_vote_msg_v3(req.request.clone());

    req.signature
        .clone()
        .verify_identity(sender, msg)
        .map_err(|_| (401, WorkerError::InvalidSignature))?;

    Ok(())
}

fn verify_airdrop_claim_req(
    req: &VerifiableClaimRequest,
) -> StdResult<(), (u16, AirdropClaimError)> {
    let msg = hon_worker_common::verifiable_claim_request_message(req.request.clone());

    req.signature
        .clone()
        .verify_identity(req.sender, msg)
        .map_err(|_| (401, AirdropClaimError::InvalidSignature))?;

    Ok(())
}

fn verify_hon_referral_req(req: &ReferralReqWithSignature) -> StdResult<(), (u16, WorkerError)> {
    let msg = hon_referral_msg(req.request.clone());

    req.signature
        .clone()
        .verify_identity(req.request.referee, msg)
        .map_err(|_| (401, WorkerError::InvalidSignature))?;

    Ok(())
}

fn get_hon_game_stub<T>(ctx: &RouteContext<T>, user_principal: Principal) -> Result<Stub> {
    let game_ns = ctx.durable_object("USER_HON_GAME_STATE")?;
    let game_state_obj = game_ns.id_from_name(&user_principal.to_text())?;
    let game_stub = game_state_obj.get_stub()?;

    Ok(game_stub)
}

fn get_hon_game_stub_env(env: &Env, user_principal: Principal) -> Result<Stub> {
    let game_ns = env.durable_object("USER_HON_GAME_STATE")?;
    let game_state_obj = game_ns.id_from_name(&user_principal.to_text())?;
    let game_stub = game_state_obj.get_stub()?;

    Ok(game_stub)
}

async fn place_hot_or_not_vote(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    };

    let user_principal = parse_principal!(ctx, "user_principal");

    let req: HoNGameVoteReq = serde_json::from_str(&req.text().await?)?;
    if let Err((code, err)) = verify_hon_game_req(user_principal, &req) {
        return err_to_resp(code, err);
    };

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req = VoteRequestWithSentiment {
        request: req.request,
        sentiment: req.fetched_sentiment,
        post_creator: req.post_creator,
    };

    let req = Request::new_with_init(
        "http://fake_url.com/vote",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

async fn place_hot_or_not_vote_v2(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    };

    let user_principal = parse_principal!(ctx, "user_principal");

    let req: HoNGameVoteReq = serde_json::from_str(&req.text().await?)?;
    if let Err((code, err)) = verify_hon_game_req(user_principal, &req) {
        return err_to_resp(code, err);
    };

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req = VoteRequestWithSentiment {
        request: req.request,
        sentiment: req.fetched_sentiment,
        post_creator: req.post_creator,
    };

    let req = Request::new_with_init(
        "http://fake_url.com/vote_v2",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

async fn place_hot_or_not_vote_v3(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    };

    let user_principal = parse_principal!(ctx, "user_principal");

    let req: HoNGameVoteReqV3 = serde_json::from_str(&req.text().await?)?;
    if let Err((code, err)) = verify_hon_game_req_v3(user_principal, &req) {
        return err_to_resp(code, err);
    };

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req = VoteRequestWithSentimentV3 {
        request: req.request,
        sentiment: req.fetched_sentiment,
        post_creator: req.post_creator,
    };

    let req = Request::new_with_init(
        "http://fake_url.com/v3/vote",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

async fn user_sats_balance(ctx: RouteContext<()>, use_v2: bool) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let endpoint = if use_v2 { "v2/balance" } else { "balance" };

    let res = game_stub
        .fetch_with_str(&format!("http://fake_url.com/{endpoint}"))
        .await?;

    Ok(res)
}

async fn last_airdrop_claimed_at(ctx: RouteContext<()>) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let res = game_stub
        .fetch_with_str("http://fake_url.com/last_airdrop_claimed_at")
        .await?;

    Ok(res)
}

async fn game_info(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req_data: GameInfoReq = req.json().await?;

    let req = Request::new_with_init(
        "http://fake_url.com/game_info",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req_data)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

async fn paginated_games(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req_data: PaginatedGamesReq = req.json().await?;

    let req = Request::new_with_init(
        "http://fake_url.com/games",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req_data)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

async fn game_info_v3(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req_data: GameInfoReqV3 = req.json().await?;

    let req = Request::new_with_init(
        "http://fake_url.com/v3/game_info",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req_data)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

async fn paginated_games_v3(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req_data: PaginatedGamesReq = req.json().await?;

    let req = Request::new_with_init(
        "http://fake_url.com/v3/games",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req_data)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

fn verify_hon_withdraw_req(req: &HoNGameWithdrawReq) -> StdResult<(), (u16, WorkerError)> {
    let msg = hon_game_withdraw_msg(&req.request);

    req.signature
        .clone()
        .verify_identity(req.request.receiver, msg)
        .map_err(|_| (401, WorkerError::InvalidSignature))?;

    Ok(())
}

async fn claim_airdrop(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    };
    let req: VerifiableClaimRequest = serde_json::from_str(&req.text().await?)?;
    if let Err(e) = verify_airdrop_claim_req(&req) {
        return err_to_resp(e.0, e.1);
    }

    let user_principal = parse_principal!(ctx, "user_principal");

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req = Request::new_with_init(
        "http://fake_url.com/claim_airdrop",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req.amount)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

async fn withdraw_sats(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    };
    let req: HoNGameWithdrawReq = serde_json::from_str(&req.text().await?)?;
    if let Err(e) = verify_hon_withdraw_req(&req) {
        return err_to_resp(e.0, e.1);
    }

    let game_stub = get_hon_game_stub(&ctx, req.request.receiver)?;

    let req = Request::new_with_init(
        "http://fake_url.com/withdraw",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req.request)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

async fn referral_reward(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    };

    let req_with_sig: ReferralReqWithSignature = serde_json::from_str(&req.text().await?)?;
    if let Err((code, err)) = verify_hon_referral_req(&req_with_sig) {
        return err_to_resp(code, err);
    }

    let req = req_with_sig.request;

    let state_backend = StateBackend::new(&ctx.env)?;
    let is_referee_registered = state_backend
        .is_user_registered(req.referee_canister, req.referee)
        .await?;
    if !is_referee_registered {
        return err_to_resp(
            400,
            WorkerError::Internal("Referee is not registered".to_string()),
        );
    }

    let referee_game_stub = get_hon_game_stub(&ctx, req.referee)?;
    let add_referee_signup_reward_req = Request::new_with_init(
        "http://fake_url.com/add_referee_signup_reward_v2",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req)?
            .build(),
    )?;

    let mut add_referee_signup_reward_res = referee_game_stub
        .fetch_with_request(add_referee_signup_reward_req)
        .await?;
    if add_referee_signup_reward_res.status_code() != 200 {
        return err_to_resp(
            add_referee_signup_reward_res.status_code(),
            WorkerError::Internal(add_referee_signup_reward_res.text().await?),
        );
    }

    let referrer_game_stub = get_hon_game_stub(&ctx, req.referrer)?;
    let add_referrer_reward_req = Request::new_with_init(
        "http://fake_url.com/add_referrer_reward_v2",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req)?
            .build(),
    )?;

    let mut add_referrer_reward_res = referrer_game_stub
        .fetch_with_request(add_referrer_reward_req)
        .await?;
    if add_referrer_reward_res.status_code() != 200 {
        return err_to_resp(
            add_referrer_reward_res.status_code(),
            WorkerError::Internal(add_referrer_reward_res.text().await?),
        );
    }

    let notif_client = NotificationClient::new(
        ctx.env
            .secret("YRAL_METADATA_USER_NOTIFICATION_API_KEY")?
            .to_string(),
    );
    notif_client
        .send_notification(
            NotificationType::ReferrerReferralReward {
                referee_principal: req.referee,
                amount: req.amount,
            },
            Some(req.referrer),
        )
        .await;

    // send sample success response
    let res = Response::from_json(&json!({
        "success": true,
        "message": "Referral created successfully"
    }))?;

    Ok(res)
}

async fn referral_paginated_history(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req: PaginatedReferralsReq = serde_json::from_str(&req.text().await?)?;

    let req = Request::new_with_init(
        "http://fake_url.com/referral_history",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req)?
            .build(),
    )?;

    let res = game_stub.fetch_with_request(req).await?;

    Ok(res)
}

async fn update_sats_balance(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    };

    let user_principal = parse_principal!(ctx, "user_principal");

    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let req_data: SatsBalanceUpdateRequest = serde_json::from_str(&req.text().await?)?;

    let req = Request::new_with_init(
        "http://fake_url.com/update_balance",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req_data)?
            .build(),
    )?;

    game_stub.fetch_with_request(req).await
}

async fn update_sats_balance_v2(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    };

    let user_principal = parse_principal!(ctx, "user_principal");
    let game_stub = get_hon_game_stub_env(&ctx.env, user_principal)?;

    let req_data: SatsBalanceUpdateRequestV2 = serde_json::from_str(&req.text().await?)?;

    let req = Request::new_with_init(
        "http://fake_url.com/v2/update_balance",
        RequestInitBuilder::default()
            .method(Method::Post)
            .json(&req_data)?
            .build(),
    )?;

    game_stub.fetch_with_request(req).await
}

async fn migrate_games(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Err((msg, code)) = verify_jwt_from_header(JWT_PUBKEY, JWT_AUD.into(), &req) {
        return Response::error(msg, code);
    }
    let user_principal = parse_principal!(ctx, "user_principal");
    let game_stub = get_hon_game_stub(&ctx, user_principal)?;
    let req = Request::new_with_init(
        "http://fake_url.com/migrate",
        RequestInitBuilder::default().method(Method::Post).build(),
    )?;
    game_stub.fetch_with_request(req).await
}

async fn estabilish_balance_ws(ctx: RouteContext<()>) -> Result<Response> {
    let user_principal = parse_principal!(ctx, "user_principal");
    let game_stub = get_hon_game_stub(&ctx, user_principal)?;

    let mut headers = Headers::new();
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
            user_sats_balance(ctx, false)
        })
        .get_async("/v2/balance/:user_principal", |_req, ctx| {
            user_sats_balance(ctx, true)
        })
        .post_async("/game_info/:user_principal", game_info)
        .post_async("/games/:user_principal", |req, ctx| {
            paginated_games(req, ctx)
        })
        .post_async("/vote/:user_principal", |req, ctx| {
            place_hot_or_not_vote(req, ctx)
        })
        .post_async("/vote_v2/:user_principal", |req, ctx| {
            place_hot_or_not_vote_v2(req, ctx)
        })
        .post_async("/v3/vote/:user_principal", |req, ctx| {
            place_hot_or_not_vote_v3(req, ctx)
        })
        .post_async("/v3/game_info/:user_principal", game_info_v3)
        .post_async("/v3/games/:user_principal", |req, ctx| {
            paginated_games_v3(req, ctx)
        })
        .post_async("/claim_airdrop/:user_principal", |req, ctx| {
            claim_airdrop(req, ctx)
        })
        .get_async("/last_airdrop_claimed_at/:user_principal", |_req, ctx| {
            last_airdrop_claimed_at(ctx)
        })
        .post_async("/withdraw", withdraw_sats)
        .post_async("/referral_reward", referral_reward)
        .post_async(
            "/referral_history/:user_principal",
            referral_paginated_history,
        )
        .post_async("/update_balance/:user_principal", update_sats_balance)
        .post_async("/v2/update_balance/:user_principal", update_sats_balance_v2)
        .post_async("/migrate/:user_principal", migrate_games)
        .get_async("/ws/balance/:user_principal", |_req, ctx| {
            estabilish_balance_ws(ctx)
        })
        .options("/*catchall", |_, _| Response::empty())
        .run(req, env)
        .await?;

    res.with_cors(&cors_policy())
}
