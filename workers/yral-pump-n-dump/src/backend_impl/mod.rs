mod mock;
mod real;

use candid::{Nat, Principal};
use enum_dispatch::enum_dispatch;
use mock::{MockWsBackend, NoOpGameBackend, NoOpUserState};
use worker::{Env, Result};
use worker_utils::environment::{env_kind, RunEnv};
use yral_canisters_client::individual_user_template::{
    BalanceInfo, BettingStatus, PumpNDumpStateDiff,
};

use crate::admin_cans::AdminCans;

#[enum_dispatch]
pub(crate) trait GameBackendImpl {
    async fn add_dollr_to_liquidity_pool(
        &self,
        user_canister: Principal,
        token_root: Principal,
        amount: Nat,
    ) -> Result<()>;
}

#[enum_dispatch]
pub(crate) trait UserStateBackendImpl {
    async fn game_balance(&self, user_canister: Principal) -> Result<BalanceInfo>;

    async fn game_balance_v2(&self, user_canister: Principal) -> Result<BalanceInfo>;

    async fn reconcile_user_state(
        &self,
        user_canister: Principal,
        completed_games: Vec<PumpNDumpStateDiff>,
    ) -> Result<()>;

    async fn redeem_gdollr(&self, user_canister: Principal, amount: Nat) -> Result<()>;

    // async fn bet_on_hot_or_not_post(
    //     &self,
    //     user_canister: Principal,
    //     args: PlaceBetArg,
    // ) -> Result<std::result::Result<BettingStatus, BetOnCurrentlyViewingPostError>>;

    async fn game_count(&self, user_canister: Principal) -> Result<u64>;

    async fn net_earnings(&self, user_canister: Principal) -> Result<Nat>;

    async fn canister_controller(&self, user_canister: Principal) -> Result<Principal>;

    async fn dolr_balance(&self, user_index: Principal) -> Result<Nat>;

    async fn dolr_transfer(&self, to: Principal, amount: Nat) -> Result<()>;
}

#[enum_dispatch]
pub(crate) trait WsBackendImpl {
    async fn user_principal_to_user_canister(
        &self,
        user_principal: Principal,
    ) -> Result<Option<Principal>>;

    async fn validate_token(
        &self,
        token_root: Principal,
        token_creator_canister: Principal,
    ) -> Result<bool>;
}

#[derive(Clone)]
#[enum_dispatch(GameBackendImpl)]
pub enum GameBackend {
    Real(AdminCans),
    Mock(NoOpGameBackend),
}

impl GameBackend {
    pub fn new(env: &Env) -> Result<Self> {
        if env_kind() == RunEnv::Mock {
            Ok(GameBackend::Mock(NoOpGameBackend))
        } else {
            AdminCans::new(env).map(Self::Real)
        }
    }
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

#[derive(Clone)]
#[enum_dispatch(WsBackendImpl)]
pub enum WsBackend {
    Real(AdminCans),
    Mock(MockWsBackend),
}

impl WsBackend {
    pub fn new(env: &Env) -> Result<Self> {
        if env_kind() == RunEnv::Mock {
            Ok(WsBackend::Mock(MockWsBackend))
        } else {
            AdminCans::new(env).map(Self::Real)
        }
    }
}
