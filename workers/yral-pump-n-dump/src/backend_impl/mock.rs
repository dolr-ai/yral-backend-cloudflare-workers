use candid::{Nat, Principal};
use worker::{console_debug, Result};
use yral_canisters_client::individual_user_template::{
    BalanceInfo, BettingStatus, PumpNDumpStateDiff,
};

use crate::consts::GDOLLR_TO_E8S;

use super::{GameBackendImpl, UserStateBackendImpl, WsBackendImpl};

#[derive(Clone)]
pub struct NoOpGameBackend;

impl GameBackendImpl for NoOpGameBackend {
    async fn add_dollr_to_liquidity_pool(
        &self,
        _user_canister: Principal,
        _token_root: Principal,
        _amount: Nat,
    ) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct NoOpUserState;

const FAKE_BALANCE: u64 = 100 * GDOLLR_TO_E8S;

impl UserStateBackendImpl for NoOpUserState {
    async fn game_balance(&self, _user_canister: Principal) -> Result<BalanceInfo> {
        Ok(BalanceInfo {
            net_airdrop_reward: 0u32.into(),
            balance: FAKE_BALANCE.into(),
            withdrawable: FAKE_BALANCE.into(),
        })
    }

    async fn game_balance_v2(&self, user_canister: Principal) -> Result<BalanceInfo> {
        self.game_balance(user_canister).await
    }

    async fn reconcile_user_state(
        &self,
        _user_canister: Principal,
        _games: Vec<PumpNDumpStateDiff>,
    ) -> Result<()> {
        Ok(())
    }

    async fn redeem_gdollr(&self, _user_canister: Principal, _amount: Nat) -> Result<()> {
        Ok(())
    }

    async fn game_count(&self, _user_canister: Principal) -> Result<u64> {
        Ok(10)
    }

    async fn net_earnings(&self, _user_canister: Principal) -> Result<Nat> {
        Ok(FAKE_BALANCE.into())
    }

    async fn dolr_balance(&self, _user_index: Principal) -> Result<Nat> {
        Ok(Nat::from(u64::MAX))
    }

    async fn canister_controller(&self, user_canister: Principal) -> Result<Principal> {
        Ok(user_canister)
    }

    async fn dolr_transfer(&self, _to: Principal, _amount: Nat) -> Result<()> {
        Ok(())
    }

    // async fn bet_on_hot_or_not_post(
    //     &self,
    //     user_canister: Principal,
    //     args: PlaceBetArg,
    // ) -> Result<std::result::Result<BettingStatus, BetOnCurrentlyViewingPostError>> {
    //     console_debug!("Dummy call to canister for hon bet");
    //     console_debug!("  user_canister: {user_canister}");
    //     console_debug!("  args: {args:#?}");

    //     Ok(Ok(BettingStatus::BettingClosed))
    // }
}

#[derive(Clone)]
pub struct MockWsBackend;

impl WsBackendImpl for MockWsBackend {
    async fn user_principal_to_user_canister(
        &self,
        user_principal: Principal,
    ) -> Result<Option<Principal>> {
        Ok(Some(user_principal))
    }

    async fn validate_token(
        &self,
        _token_root: Principal,
        _token_creator: Principal,
    ) -> Result<bool> {
        Ok(true)
    }
}
