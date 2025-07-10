use candid::Principal;
use futures::future::join_all;
use worker::{console_log, Env, Request, Response, Result, RouteContext};

use crate::{
    individual_user_canister::backup_restore::backup::individual_user_backup_impl,
    utils::{create_agent, get_all_available_cansiters_from_networks},
};

pub async fn individual_user_bulk_backup(env: &Env) -> Result<Response> {
    let agent = create_agent(env).await;

    let cans_ids: Vec<Principal> = get_all_available_cansiters_from_networks(&agent).await?;

    if cans_ids.is_empty() {
        console_log!("No canister IDs found for backup.");
        return Response::ok("No canister IDs found for backup.");
    }

    let backup_futures: Vec<_> = cans_ids
        .into_iter()
        .map(|cans_id| {
            let env = env.clone();
            let agent = agent.clone();
            async move { individual_user_backup_impl(env, agent, cans_id).await }
        })
        .collect();

    let results = join_all(backup_futures).await;

    let mut success_count = 0;
    let mut failure_count = 0;
    for result in results {
        match result {
            Ok(_) => success_count += 1,
            Err(err) => {
                failure_count += 1;
                console_log!("Backup error: {}", err);
            }
        }
    }

    console_log!(
        "Backup completed. Success: {}, Failures: {}",
        success_count,
        failure_count
    );

    Response::ok(format!(
        "Backup completed. Success: {success_count}, Failures: {failure_count}",
    ))
}

pub async fn individual_user_bulk_backup_handler(
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

    individual_user_bulk_backup(&ctx.env).await
}
