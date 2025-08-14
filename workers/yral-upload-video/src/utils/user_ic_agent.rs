use std::{collections::HashMap, error::Error};

use ic_agent::{identity::DelegatedIdentity, Agent};

use super::types::{DelegatedIdentityWire, DELEGATED_IDENTITY_KEY};

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
