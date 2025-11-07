use std::error::Error;

use axum::http::HeaderMap;
use candid::Principal;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use worker::console_log;

use crate::{
    server_impl::upload_video_to_canister::upload_ai_video_to_canister_as_draft,
    utils::types::{NotifyRequestPayload, POST_ID, USER_ID},
};

pub fn verify_webhook_signature(
    webhook_secret_key: String,
    webhook_signature: &str,
    req_data: String,
) -> Result<(), Box<dyn Error>> {
    let mut time_and_signature = webhook_signature.split(",");

    let time = time_and_signature
        .next()
        .ok_or("time not found in web signature")?
        .split("=")
        .last()
        .ok_or("invalid time header format")?;

    let signature = time_and_signature
        .next()
        .ok_or("signature not found in web signature")?
        .split("=")
        .last()
        .ok_or("invalid signature header format")?;

    let input_str = format!("{time}.{req_data}");

    type HmacSha256 = Hmac<Sha256>;

    let mut hmac = HmacSha256::new_from_slice(webhook_secret_key.as_bytes())?;

    hmac.update(input_str.as_bytes());

    let mac_result = hmac.finalize();
    let result_str = mac_result.into_bytes();
    let digest = hex::encode(result_str);

    if digest.eq(&signature) {
        Ok(())
    } else {
        Err("Invalid webhook signature".into())
    }
}

pub async fn notify_video_upload_impl(
    admin_agent: &ic_agent::Agent,
    req_data: String,
    headers: HeaderMap,
    webhook_secret_key: String,
) -> Result<(), Box<dyn Error>> {
    let webhook_signature = headers
        .get("Webhook-Signature")
        .ok_or("Signature not found")?
        .to_str()?;

    let notify_req_paylod: NotifyRequestPayload = serde_json::from_str(&req_data)?;

    verify_webhook_signature(webhook_secret_key, webhook_signature, req_data)?;

    if notify_req_paylod
        .status
        .state
        .is_some_and(|s| s.eq("error"))
    {
        return Err(notify_req_paylod
            .status
            .err_reason_text
            .unwrap_or("unknown error while processing video".into())
            .into());
    }

    let Some(post_id) = notify_req_paylod.meta.get(POST_ID) else {
        console_log!("Post ID identity not found. Not generated from ai video generation");
        return Ok(());
    };

    let Some(user_principal_str) = notify_req_paylod.meta.get(USER_ID) else {
        console_log!("User ID not found. Not generated from ai video generation");
        return Ok(());
    };

    let user_principal = Principal::from_text(user_principal_str)?;

    let video_uid = notify_req_paylod.uid;
    upload_ai_video_to_canister_as_draft(admin_agent, user_principal, post_id.clone(), video_uid)
        .await?;

    //TODO send notifications to user about the video uplaod.

    Ok(())
}
