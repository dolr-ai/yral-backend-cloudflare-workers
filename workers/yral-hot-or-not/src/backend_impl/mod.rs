mod mock;
mod real;

use candid::Principal;
use enum_dispatch::enum_dispatch;
use mock::NoOpUserState;
use worker::{Env, Result};
use worker_utils::environment::{env_kind, RunEnv};

use crate::admin_cans::AdminCans;

#[enum_dispatch]
pub(crate) trait UserStateBackendImpl {
    async fn is_user_registered(
        &self,
        user_canister: Principal,
        user_principal: Principal,
    ) -> Result<bool>;
}

#[derive(Clone)]
#[enum_dispatch(UserStateBackendImpl)]
pub enum StateBackend {
    Real(AdminCans),
    Mock(NoOpUserState),
}

impl StateBackend {
    pub fn new(env: &Env) -> Result<Self> {
        if env_kind() == RunEnv::Mock {
            Ok(StateBackend::Mock(NoOpUserState))
        } else {
            AdminCans::new(env).map(Self::Real)
        }
    }
}
