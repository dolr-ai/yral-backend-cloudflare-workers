use candid::Principal;
use ic_agent::identity::Secp256k1Identity;
use k256::SecretKey;
use worker::{Env, Result};
use worker_utils::{
    environment::{env_kind, RunEnv},
    icp::agent_wrapper::AgentWrapper,
};
use yral_canisters_client::individual_user_template::IndividualUserTemplate;

use crate::consts::ADMIN_LOCAL_SECP_SK;

#[derive(Clone)]
pub struct AdminCans {
    pub agent: AgentWrapper,
}

impl AdminCans {
    pub fn new(env: &Env) -> Result<Self> {
        let agent;

        match env_kind() {
            RunEnv::Local => {
                let id = Secp256k1Identity::from_private_key(
                    SecretKey::from_bytes(&ADMIN_LOCAL_SECP_SK.into()).unwrap(),
                );
                agent = AgentWrapper::new(id);
            }
            RunEnv::Remote => {
                let admin_pem = env.secret("BACKEND_ADMIN_KEY")?.to_string();
                let id = Secp256k1Identity::from_pem(admin_pem.as_bytes())
                    .map_err(|e| worker::Error::RustError(e.to_string()))?;
                agent = AgentWrapper::new(id);
            }
            RunEnv::Mock => panic!("trying to use ic-agent in mock env"),
        };

        Ok(Self { agent })
    }

    pub async fn individual_user(&self, user_canister: Principal) -> IndividualUserTemplate<'_> {
        IndividualUserTemplate(user_canister, self.agent.get().await)
    }
}
