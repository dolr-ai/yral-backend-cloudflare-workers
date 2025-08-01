use std::{collections::HashMap, error::Error};

use ic_agent::{identity::DelegatedIdentity, Agent};

use super::{
    individual_user_canister::Service as UserCanisterService,
    types::{DelegatedIdentityWire, DELEGATED_IDENTITY_KEY},
};
use yral_metadata_client::MetadataClient;

#[derive(Clone)]
pub struct UserICAgent<'a> {
    // need to find the canister id as well.
    pub user_canister_service: UserCanisterService<'a>,
}

impl<'a> UserICAgent<'a> {
    #[allow(dead_code)]
    async fn new(ic_agent: &'a Agent) -> Result<Self, Box<dyn Error>> {
        let user_principal = ic_agent.get_principal()?;

        let yral_metadata_client = MetadataClient::default();

        let user_metadata = yral_metadata_client
            .get_user_metadata_v2(user_principal.to_text())
            .await?
            .ok_or("user canister not found")?;

        Ok(Self {
            user_canister_service: UserCanisterService(user_metadata.user_canister_id, ic_agent),
        })
    }
}

pub fn create_ic_agent_from_meta(meta: &HashMap<String, String>) -> Result<Agent, Box<dyn Error>> {
    let delegated_identity_string = meta
        .get(DELEGATED_IDENTITY_KEY)
        .ok_or("delegated identity not found")?;

    let delegated_identity_wire: DelegatedIdentityWire =
        serde_json::from_str(delegated_identity_string)?;

    let delegated_identity = DelegatedIdentity::try_from(delegated_identity_wire)?;
    let ic_agent = Agent::builder()
        .with_identity(delegated_identity)
        .with_url("https://ic0.app/")
        .build()?;

    Ok(ic_agent)
}
