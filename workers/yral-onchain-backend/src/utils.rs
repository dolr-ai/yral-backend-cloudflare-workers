use crate::{platform_ochestrator::PLATFORM_ORCHESTRATOR_CANS_ID, JsValue};
use base64::{prelude::BASE64_STANDARD, Engine};
use candid::{CandidType, Decode, Encode, Principal};
use ic_agent::Agent;
use serde::Deserialize;
use serde_json::json;
use worker::{Env, Error, Method, Request, RequestInit, Response, Result, Stub};

#[derive(CandidType, Deserialize, Debug)]
pub struct UserProfileDetailsForFrontend {
    pub unique_user_name: Option<String>,
    pub lifetime_earnings: u64,
    pub following_count: u64,
    pub profile_picture_url: Option<String>,
    pub display_name: Option<String>,
    pub principal_id: Principal,
    pub profile_stats: UserProfileGlobalStats,
    pub followers_count: u64,
    pub referrer_details: Option<UserCanisterDetails>,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct UserProfileGlobalStats {
    pub hot_bets_received: u64,
    pub not_bets_received: u64,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct UserCanisterDetails {
    pub user_canister_id: Principal,
    pub profile_owner: Principal,
}

pub async fn get_user_principal_from_canister(
    agent: &Agent,
    user_canister: Principal,
) -> Result<Option<Principal>> {
    let response = Decode!(
        &agent
            .query(&user_canister, "get_profile_details_v2")
            .with_arg(Encode!().map_err(|e| e.to_string())?)
            .call()
            .await
            .map_err(|e| e.to_string())?,
        UserProfileDetailsForFrontend
    )
    .map_err(|e| e.to_string())?;
    Ok(Some(response.principal_id))
}

async fn get_all_available_canisters(
    agent: &Agent,
    user_index: Principal,
) -> Result<Vec<Principal>> {
    let res = agent
        .query(&user_index, "get_user_canister_list")
        .with_arg(Encode!().unwrap())
        .await
        .map_err(|e| e.to_string())?;
    Ok(Decode!(&res, Vec<Principal>).map_err(|e| e.to_string())?)
}

pub async fn create_agent(env: &Env) -> Agent {
    #[cfg(feature = "local")]
    {
        _ = env;
        let agent = Agent::builder()
            .with_url("http://localhost:4943")
            .build()
            .unwrap();
        agent.fetch_root_key().await.unwrap();
        agent
    }

    #[cfg(not(feature = "local"))]
    {
        let pk = env
            .secret("RECLAIM_CANISTER_PEM")
            .expect("$RECLAIM_CANISTER_PEM is not set")
            .to_string();

        let identity = ic_agent::identity::BasicIdentity::from_pem(
            stringreader::StringReader::new(pk.as_str()),
        )
        .unwrap();

        Agent::builder()
            .with_url("https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.ic0.app") // https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.ic0.app/
            .with_identity(identity)
            .build()
            .unwrap()
    }
}

pub async fn get_all_user_indexes(agent: &Agent) -> Result<Vec<Principal>> {
    let platfrom_orchestrator_cans_id =
        Principal::from_text(PLATFORM_ORCHESTRATOR_CANS_ID).unwrap();

    Ok(Decode!(
        &agent
            .query(
                &platfrom_orchestrator_cans_id,
                "get_all_subnet_orchestrators"
            )
            .with_arg(Encode!().unwrap())
            .call()
            .await
            .map_err(|e| e.to_string())?,
        Vec<Principal>
    )
    .map_err(|e| e.to_string())?)
}

pub async fn get_all_available_cansiters_from_networks(agent: &Agent) -> Result<Vec<Principal>> {
    let user_indexes = get_all_user_indexes(agent).await?;

    let mut ret = vec![];
    for user_index in user_indexes {
        let cans = get_all_available_canisters(agent, user_index).await?;
        ret.extend(cans);
    }

    Ok(ret)
}

pub async fn backup_impl(
    agent: &Agent,
    canister_id: Principal,
    stub: Stub,
    env: &Env,
) -> Result<Response> {
    let response = agent
        .update(&canister_id, "save_snapshot_json")
        .with_arg(Encode!().unwrap())
        .call_and_wait()
        .await
        .map_err(|e| Error::RustError(e.to_string()))?;

    let snapshot_length: u32 =
        Decode!(&response, u32).map_err(|e| Error::RustError(e.to_string()))?;

    let mut data: Vec<u8> = Vec::new();
    let mut offset: u64 = 0;
    let chunk_size: u64 = 1_000_000; // 1 MB

    while offset < snapshot_length as u64 {
        let length = std::cmp::min(chunk_size, snapshot_length as u64 - offset);

        let response = agent
            .query(&canister_id, "download_snapshot")
            .with_arg(Encode!(&offset, &length).unwrap())
            .call()
            .await
            .map_err(|e| Error::RustError(e.to_string()))?;

        let chunk: Vec<u8> =
            Decode!(&response, Vec<u8>).map_err(|e| Error::RustError(e.to_string()))?;
        data.extend_from_slice(&chunk);
        offset += length;
    }

    let data = BASE64_STANDARD.encode(data);
    let mut init = RequestInit::new();
    init.with_method(Method::Put)
        .with_body(Some(JsValue::from(&data)));

    let init_req = Request::new_with_init(
        &format!(
            "{}/backup",
            env.var("API_HOST")
                .map(|h| h.to_string())
                .unwrap_or("http://localhost:8787".to_string())
        ),
        &init,
    )?; // doesnt really matter what url you pass in as long as the correct param is passed like /restore or /backup
    stub.fetch_with_request(init_req).await
}

pub async fn restore_impl(canister_id: Principal, stub: Stub, env: &Env) -> Result<Response> {
    let mut init = RequestInit::new();
    init.with_method(worker::Method::Put);

    init.with_body(Some(JsValue::from_str(
        &json!({
            "canister_id": canister_id
        })
        .to_string(),
    )));

    let new_req = Request::new_with_init(
        &format!(
            "{}/restore",
            env.var("API_HOST")
                .map(|h| h.to_string())
                .unwrap_or("http://localhost:8787".to_string())
        ),
        &init,
    )?; // doesnt really matter what url you pass in as long as the correct param is passed like /restore or /backup

    stub.fetch_with_request(new_req).await
}
