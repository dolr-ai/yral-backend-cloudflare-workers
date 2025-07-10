use candid::{CandidType, Decode, Encode, Principal};
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenListItem {
    pub user_id: String,
    pub name: String,
    pub token_name: String,
    pub token_symbol: String,
    pub logo: String,
    pub description: String,
    pub created_at: String,
    pub formatted_created_at: String,
    pub link: String,
    #[serde(default)]
    pub is_nsfw: bool,
}

async fn fetch_documents(env: &Env) -> Result<Vec<TokenListItem>> {
    let firestore_url = format!(
        "https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/tokens",
        project_id = env.var("FIREBASE_PROJECT_ID")?,
    );

    let client = reqwest::Client::new();
    let api_key = env.secret("FIREBASE_API_KEY")?.to_string();
    let response = client
        .get(firestore_url)
        .query(&[("key", &api_key)])
        .send()
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if response.status().is_success() {
        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        let documents = data["documents"]
            .as_array()
            .ok_or_else(|| Error::from("No documents found"))?;

        let firestore_entries: Vec<TokenListItem> = documents
            .iter()
            .filter_map(|doc| serde_json::from_value(doc["fields"].clone()).ok())
            .collect();

        Ok(firestore_entries)
    } else {
        Err(Error::from(format!(
            "Failed to fetch documents: {}",
            response.status()
        )))
    }
}

async fn create_agent() -> Result<Agent> {
    let agent = Agent::builder()
        .with_url("https://ic0.app")
        .build()
        .map_err(|e| Error::from(e.to_string()))?;
    Ok(agent)
}

#[derive(Deserialize)]
struct UserMetadata {
    user_canister_id: Principal,
}

async fn get_user_canister(_agent: &Agent, user_principal: Principal) -> Result<Option<Principal>> {
    let client = reqwest::Client::new();
    let metadata_url = format!(
        "https://yral-metadata.dev/metadata/{}",
        user_principal.to_text()
    );

    let response = client
        .get(metadata_url)
        .send()
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    if response.status().is_success() {
        let metadata: Option<UserMetadata> = response
            .json()
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        Ok(metadata.map(|m| m.user_canister_id))
    } else {
        Err(Error::from(format!(
            "Failed to fetch user metadata: {}",
            response.status()
        )))
    }
}

#[derive(Deserialize, CandidType)]
struct DeployedCdaoCanisters {
    root: Principal,
    // ... other fields omitted as they're not needed
}

async fn get_deployed_canisters(
    agent: &Agent,
    user_canister: Principal,
) -> Result<Option<Vec<Principal>>> {
    let response = agent
        .query(&user_canister, "deployed_cdao_canisters")
        .with_arg(Encode!().map_err(|e| Error::from(e.to_string()))?)
        .call()
        .await
        .map_err(|e| Error::from(e.to_string()))?;

    let deployed_canisters = Decode!(response.as_slice(), Vec<DeployedCdaoCanisters>)
        .map_err(|e| Error::from(e.to_string()))?;

    let root_principals: Vec<Principal> = deployed_canisters.iter().map(|dc| dc.root).collect();

    Ok(Some(root_principals))
}

async fn find_invalid_tokens(env: &Env) -> Result<Vec<TokenListItem>> {
    let firestore_entries = fetch_documents(env).await?;
    let agent = create_agent().await?;
    let mut invalid_tokens = Vec::new();

    for entry in firestore_entries {
        if let Some(token_principal) = entry
            .link
            .split('/')
            .next_back()
            .and_then(|id| Principal::from_text(id).ok())
        {
            if let Ok(user_principal) = Principal::from_text(&entry.user_id) {
                if let Ok(Some(user_canister)) = get_user_canister(&agent, user_principal).await {
                    if let Ok(Some(deployed_canisters)) =
                        get_deployed_canisters(&agent, user_canister).await
                    {
                        if !deployed_canisters.contains(&token_principal) {
                            invalid_tokens.push(entry);
                        }
                        continue;
                    }
                }
            }
            //what if check fails
            invalid_tokens.push(entry);
        } else {
            // invalid_token canister ID
            invalid_tokens.push(entry);
        }
    }

    Ok(invalid_tokens)
}

async fn delete_invalid_tokens(env: &Env, invalid_tokens: &[TokenListItem]) -> Result<Vec<String>> {
    let client = reqwest::Client::new();
    let api_key = env.secret("FIREBASE_API_KEY")?.to_string();
    let project_id = env.var("FIREBASE_PROJECT_ID")?.to_string();
    let mut deleted_tokens = Vec::new();

    for token in invalid_tokens {
        let token_id = token
            .link
            .split('/')
            .next_back()
            .ok_or_else(|| Error::from("Invalid token link format"))?;

        let delete_url = format!(
            "https://firestore.googleapis.com/v1/projects/{project_id}/databases/(default)/documents/tokens/{token_id}",      
        );

        let response = client
            .delete(&delete_url)
            .query(&[("key", &api_key)])
            .send()
            .await
            .map_err(|e| Error::from(e.to_string()))?;

        if response.status().is_success() {
            deleted_tokens.push(token_id.to_string());
        }
    }

    Ok(deleted_tokens)
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get_async("/", |req: Request, ctx: RouteContext<()>| {
            async move {
                // Getting env from context instead of cloning
                let env = ctx.env;

                // Verify authorization
                let auth_header = req
                    .headers()
                    .get("Authorization")
                    .map_err(|e| Error::from(e.to_string()))?
                    .ok_or_else(|| Error::from("Missing authorization header"))?;

                let worker_token = env.secret("WORKER_AUTH_TOKEN")?.to_string();
                if auth_header != worker_token {
                    return Response::error("Unauthorized", 401);
                }

                // Process the request
                let invalid_tokens = find_invalid_tokens(&env)
                    .await
                    .map_err(|e| Error::from(format!("Error finding invalid tokens: {e}")))?;

                let deletion_result = if !invalid_tokens.is_empty() {
                    delete_invalid_tokens(&env, &invalid_tokens)
                        .await
                        .map_err(|e| Error::from(format!("Deletion error: {e}")))?
                } else {
                    Vec::new()
                };

                let response_data = serde_json::json!({
                    "invalid_tokens": invalid_tokens,
                    "count": invalid_tokens.len(),
                    "deleted_tokens": deletion_result,
                    "deleted_count": deletion_result.len()
                });

                Response::from_json(&response_data).map_err(|e| Error::from(e.to_string()))
            }
        })
        .run(req, env)
        .await
}
