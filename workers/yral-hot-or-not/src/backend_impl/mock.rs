use candid::Principal;
use worker::Result;

use super::UserStateBackendImpl;

#[derive(Clone)]
pub struct NoOpUserState;

impl UserStateBackendImpl for NoOpUserState {
    async fn is_user_registered(
        &self,
        _user_canister: Principal,
        _user_principal: Principal,
    ) -> Result<bool> {
        Ok(true)
    }
}
