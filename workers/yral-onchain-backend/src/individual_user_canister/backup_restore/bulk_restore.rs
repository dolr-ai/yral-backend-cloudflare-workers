use crate::utils::{create_agent, get_all_available_cansiters_from_networks};
use futures::future::join_all;
use serde_json::json;
use worker::{Request, Response, Result, RouteContext};

use super::restore::individual_user_restore_impl;

pub async fn individual_user_bulk_restore_handler(
    req: Request,
    ctx: RouteContext<()>,
) -> Result<Response> {
    let auth_header = req.headers().get("AUTH_TOKEN")?.unwrap_or_default();
    if auth_header
        != ctx
            .secret("CF_WORKER_ACCESS_OFF_CHAIN_AGENT_KEY")
            .unwrap()
            .to_string()
    {
        return Response::error("Incorrect Auth Token", 403);
    }

    let agent = create_agent(&ctx.env).await;

    let available_canisters = match get_all_available_cansiters_from_networks(&agent).await {
        Ok(cans) => cans,
        Err(e) => return Response::error(format!("Failed to fetch canisters: {e}"), 500),
    };

    let handles = available_canisters.into_iter().map(|cans| {
        // creating a request for durable object
        let agent = agent.clone();
        let env = ctx.env.clone();

        async move { individual_user_restore_impl(env, agent, cans).await }
    });

    let results = join_all(handles).await;

    let mut success_count = 0;
    let mut failure_details = Vec::new();

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(_) => success_count += 1,
            Err(e) => failure_details.push(format!("Canister {}: {:?}", i + 1, e)),
        }
    }

    let response_body = json!({
        "success_count": success_count,
        "failure_count": failure_details.len(),
        "failures": failure_details,
    });

    Response::from_json(&response_body)
}
