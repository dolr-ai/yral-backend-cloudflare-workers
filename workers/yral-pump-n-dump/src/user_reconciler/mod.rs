mod treasury;

use std::collections::HashSet;

use candid::{Nat, Principal};
use num_bigint::{BigInt, BigUint, ToBigInt};
use pump_n_dump_common::rest::{BalanceInfoResponse, CompletedGameInfo, UncommittedGameInfo};
use serde::{Deserialize, Serialize};
use treasury::DolrTreasury;
use worker::*;
use worker_utils::{
    parse_principal,
    storage::{SafeStorage, StorageCell},
};
use yral_canisters_client::individual_user_template::{
    BalanceInfo, BettingStatus, PumpNDumpStateDiff, SystemTime,
};
use yral_canisters_common::utils::vote::HonBetArg;
use yral_metrics::metrics::cents_withdrawal::CentsWithdrawal;

use crate::{
    backend_impl::{StateBackend, UserStateBackendImpl},
    consts::{GDOLLR_TO_E8S, USER_INDEX_FUND_AMOUNT, USER_STATE_RECONCILE_TIME_MS},
    utils::{metrics, CfMetricTx},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct AddRewardReq {
    pub state_diff: StateDiff,
    pub user_canister: Principal,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DecrementReq {
    pub user_canister: Principal,
    pub token_root: Principal,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClaimGdollrReq {
    pub user_canister: Principal,
    pub amount: Nat,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HotOrNotBetRequest {
    pub user_canister: Principal,
    pub args: HonBetArg,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum StateDiff {
    CompletedGame(CompletedGameInfo),
    CreatorReward(Nat),
}

impl From<StateDiff> for PumpNDumpStateDiff {
    fn from(value: StateDiff) -> Self {
        match value {
            StateDiff::CompletedGame(info) => Self::Participant(info.into()),
            StateDiff::CreatorReward(reward) => Self::CreatorReward(reward),
        }
    }
}

impl StateDiff {
    pub fn reward(&self) -> Nat {
        match self {
            Self::CompletedGame(info) => info.reward.clone(),
            Self::CreatorReward(reward) => reward.clone(),
        }
    }
}

#[durable_object]
pub struct UserEphemeralState {
    state: State,
    env: Env,
    // effective balance = on_chain_balance + off_chain_balance_delta
    off_chain_balance_delta: StorageCell<BigInt>,
    // effective earnings = on_chain_earnings + off_chain_earnings
    off_chain_earning_delta: Option<Nat>,
    user_canister: Option<Principal>,
    state_diffs: Option<Vec<StateDiff>>,
    pending_games: Option<HashSet<Principal>>,
    backend: StateBackend,
    dolr_treasury: DolrTreasury,
    metrics: CfMetricTx,
}

/// An intermediary struct that exists simply to allow serializing `SystemTime`
#[derive(Serialize, Debug, Clone)]
pub struct IntermediarySystemTime {
    pub nanos_since_epoch: u32,
    pub secs_since_epoch: u64,
}

impl From<SystemTime> for IntermediarySystemTime {
    fn from(
        SystemTime {
            nanos_since_epoch,
            secs_since_epoch,
        }: SystemTime,
    ) -> Self {
        Self {
            nanos_since_epoch,
            secs_since_epoch,
        }
    }
}

/// An intermediary struct that exists simply to allow serializing `BettingStatus`
#[derive(Serialize, Clone, Debug)]
enum IntermediaryBettingStatus {
    BettingOpen {
        number_of_participants: u8,
        ongoing_room: u64,
        ongoing_slot: u8,
        has_this_user_participated_in_this_post: Option<bool>,
        started_at: IntermediarySystemTime,
    },
    BettingClosed,
}

impl From<BettingStatus> for IntermediaryBettingStatus {
    fn from(value: BettingStatus) -> Self {
        match value {
            BettingStatus::BettingOpen {
                number_of_participants,
                ongoing_room,
                ongoing_slot,
                has_this_user_participated_in_this_post,
                started_at,
            } => Self::BettingOpen {
                number_of_participants,
                ongoing_room,
                ongoing_slot,
                has_this_user_participated_in_this_post,
                started_at: started_at.into(),
            },
            BettingStatus::BettingClosed => Self::BettingClosed,
        }
    }
}

impl UserEphemeralState {
    fn storage(&self) -> SafeStorage {
        self.state.storage().into()
    }

    async fn set_user_canister(&mut self, user_canister: Principal) -> Result<()> {
        if self.user_canister.is_some() {
            return Ok(());
        }

        self.user_canister = Some(user_canister);
        self.storage().put("user_canister", &user_canister).await?;

        Ok(())
    }

    async fn try_get_user_canister(&mut self) -> Option<Principal> {
        if let Some(user_canister) = self.user_canister {
            return Some(user_canister);
        }

        let user_canister = self.storage().get("user_canister").await.ok()??;
        self.user_canister = Some(user_canister);

        Some(user_canister)
    }

    async fn queue_settle_balance_inner(&self) -> Result<()> {
        self.state
            .storage()
            .set_alarm(USER_STATE_RECONCILE_TIME_MS)
            .await?;

        Ok(())
    }

    async fn queue_settle_balance(&self) -> Result<()> {
        let Some(alarm) = self.state.storage().get_alarm().await? else {
            return self.queue_settle_balance_inner().await;
        };
        let new_time = Date::now().as_millis() as i64 + USER_STATE_RECONCILE_TIME_MS;
        if alarm <= new_time {
            return Ok(());
        }
        self.queue_settle_balance_inner().await?;

        Ok(())
    }

    async fn off_chain_earning_delta(&mut self) -> Result<&mut Nat> {
        if self.off_chain_earning_delta.is_some() {
            return Ok(self.off_chain_earning_delta.as_mut().unwrap());
        }

        let off_chain_earning_delta = self
            .storage()
            .get::<Nat>("off_chain_earning_delta")
            .await?
            .unwrap_or_default();
        self.off_chain_earning_delta = Some(off_chain_earning_delta);
        Ok(self.off_chain_earning_delta.as_mut().unwrap())
    }

    async fn pending_games(&mut self) -> Result<&mut HashSet<Principal>> {
        if self.pending_games.is_some() {
            return Ok(self.pending_games.as_mut().unwrap());
        }

        let pending_games = self
            .storage()
            .list_with_prefix("pending-game-")
            .await
            .map(|v| v.map(|v| v.1))
            .collect::<Result<_>>()?;

        self.pending_games = Some(pending_games);
        Ok(self.pending_games.as_mut().unwrap())
    }

    async fn state_diffs(&mut self) -> Result<&mut Vec<StateDiff>> {
        if self.state_diffs.is_some() {
            return Ok(self.state_diffs.as_mut().unwrap());
        }

        let state_diffs = self
            .storage()
            .list_with_prefix("state-diff-")
            .await
            .map(|v| v.map(|v| v.1))
            .collect::<Result<_>>()?;

        self.state_diffs = Some(state_diffs);
        Ok(self.state_diffs.as_mut().unwrap())
    }

    async fn effective_balance_inner(&mut self, on_chain_balance: Nat) -> Result<Nat> {
        let mut effective_balance = on_chain_balance;
        let off_chain_delta = self
            .off_chain_balance_delta
            .read(&self.storage())
            .await?
            .clone();

        let off_chain_delta_to_subtract_biguint: BigUint = if off_chain_delta < 0u32.into() {
            (-off_chain_delta).to_biguint().unwrap()
        } else {
            (off_chain_delta).to_biguint().unwrap()
        };

        effective_balance.0 -= off_chain_delta_to_subtract_biguint.min(effective_balance.0.clone());

        Ok(effective_balance)
    }

    async fn effective_balance(&mut self, user_canister: Principal) -> Result<Nat> {
        let on_chain_balance = self.backend.game_balance(user_canister).await?;

        self.effective_balance_inner(on_chain_balance.balance).await
    }

    async fn effective_balance_info_inner(
        &mut self,
        mut bal_info: BalanceInfo,
    ) -> Result<BalanceInfo> {
        bal_info.balance = self
            .effective_balance_inner(bal_info.balance.clone())
            .await?;

        bal_info.withdrawable = if bal_info.net_airdrop_reward > bal_info.balance {
            0u32.into()
        } else {
            let bal = bal_info.balance.clone() - bal_info.net_airdrop_reward.clone();
            let treasury = self.dolr_treasury.amount(&mut self.storage()).await?;
            bal.min(treasury)
        };

        Ok(bal_info)
    }

    async fn effective_balance_info_v2(
        &mut self,
        user_canister: Principal,
    ) -> Result<BalanceInfoResponse> {
        let on_chain_bal = self.backend.game_balance_v2(user_canister).await?;
        let bal_info = self.effective_balance_info_inner_v2(on_chain_bal).await?;

        Ok(BalanceInfoResponse {
            net_airdrop_reward: bal_info.net_airdrop_reward,
            balance: bal_info.balance,
            withdrawable: bal_info.withdrawable,
        })
    }

    async fn effective_balance_info_inner_v2(
        &mut self,
        mut bal_info: BalanceInfo,
    ) -> Result<BalanceInfo> {
        bal_info.balance = self
            .effective_balance_inner(bal_info.balance.clone())
            .await?;

        let treasury = self.dolr_treasury.amount(&mut self.storage()).await?;
        bal_info.withdrawable = bal_info.withdrawable.min(treasury);

        Ok(bal_info)
    }

    async fn effective_balance_info(
        &mut self,
        user_canister: Principal,
    ) -> Result<BalanceInfoResponse> {
        let on_chain_bal = self.backend.game_balance(user_canister).await?;
        let bal_info = self.effective_balance_info_inner(on_chain_bal).await?;

        Ok(BalanceInfoResponse {
            net_airdrop_reward: bal_info.net_airdrop_reward,
            balance: bal_info.balance,
            withdrawable: bal_info.withdrawable,
        })
    }

    async fn decrement(&mut self, pending_game_root: Principal) -> Result<()> {
        let mut storage = self.storage();
        self.off_chain_balance_delta
            .update(&mut storage, |delta| *delta -= GDOLLR_TO_E8S)
            .await?;

        let inserted = self.pending_games().await?.insert(pending_game_root);
        if !inserted {
            return Ok(());
        }

        storage
            .put(
                &format!("pending-game-{pending_game_root}"),
                &pending_game_root,
            )
            .await?;

        Ok(())
    }

    async fn add_state_diff_inner(&mut self, state_diff: StateDiff) -> Result<()> {
        let reward = state_diff.reward();
        let mut storage = self.storage();
        self.off_chain_balance_delta
            .update(&mut storage, |delta| *delta += BigInt::from(reward.clone()))
            .await?;

        *self.off_chain_earning_delta().await? += reward;
        storage
            .put(
                "off_chain_earning_delta",
                self.off_chain_earning_delta().await?,
            )
            .await?;

        let state_diffs = self.state_diffs().await?;
        state_diffs.push(state_diff.clone());
        let next_idx = state_diffs.len() - 1;

        if let StateDiff::CompletedGame(ginfo) = &state_diff {
            self.pending_games().await?.remove(&ginfo.token_root);
            storage
                .delete(&format!("pending-game-{}", ginfo.token_root))
                .await?;
        }

        storage
            .put(&format!("state-diff-{next_idx}"), &state_diff)
            .await?;

        Ok(())
    }

    async fn add_state_diff(&mut self, state_diff: StateDiff) -> Result<()> {
        self.add_state_diff_inner(state_diff).await?;
        self.queue_settle_balance().await?;

        Ok(())
    }

    async fn settle_balance(&mut self, user_canister: Principal) -> Result<()> {
        let mut storage = self.storage();
        let to_settle = self.off_chain_balance_delta.read(&storage).await?.clone();

        let earnings = self.off_chain_earning_delta().await?.clone();
        self.off_chain_earning_delta = Some(0u32.into());
        storage.delete("off_chain_earning_delta").await?;

        let state_diffs = std::mem::take(self.state_diffs().await?);
        storage
            .delete_multiple(
                (0..state_diffs.len())
                    .map(|i| format!("state-diff-{i}"))
                    .collect(),
            )
            .await?;

        let mut delta_delta = BigInt::from(0u32);
        let state_diffs_conv = state_diffs
            .iter()
            .map(|diff| {
                match diff {
                    StateDiff::CompletedGame(info) => {
                        delta_delta += BigInt::from(info.pumps + info.dumps) * GDOLLR_TO_E8S;
                        delta_delta -= info.reward.clone().0.to_bigint().unwrap();
                    }
                    StateDiff::CreatorReward(rew) => {
                        delta_delta -= rew.clone().0.to_bigint().unwrap();
                    }
                }
                diff.clone().into()
            })
            .collect();

        self.off_chain_balance_delta
            .update(&mut storage, |delta| *delta += delta_delta)
            .await?;

        let res = self
            .backend
            .reconcile_user_state(user_canister, state_diffs_conv)
            .await;

        if let Err(e) = res {
            self.off_chain_balance_delta
                .set(&mut storage, to_settle)
                .await?;
            self.state_diffs = Some(state_diffs.clone());
            self.off_chain_earning_delta = Some(earnings.clone());

            storage.put("off_chain_earning_delta", &earnings).await?;

            for (i, state_diff) in state_diffs.into_iter().enumerate() {
                storage.put(&format!("state-diff-{i}"), &state_diff).await?;
            }

            return Err(e);
        }

        Ok(())
    }

    async fn check_user_index_balance(
        &mut self,
        user_canister: Principal,
        required_amount: Nat,
    ) -> Result<()> {
        let user_index = self.backend.canister_controller(user_canister).await?;
        let balance = self.backend.dolr_balance(user_index).await?;
        if balance > required_amount {
            return Ok(());
        }

        self.backend
            .dolr_transfer(user_index, USER_INDEX_FUND_AMOUNT.into())
            .await?;

        Ok(())
    }

    async fn redeem_gdollr(&mut self, user_canister: Principal, amount: Nat) -> Result<Response> {
        let mut storage = self.storage();

        self.check_user_index_balance(user_canister, amount.clone())
            .await?;
        self.dolr_treasury
            .try_consume(&mut storage, amount.clone())
            .await?;

        let res = self
            .backend
            .redeem_gdollr(user_canister, amount.clone())
            .await;
        match res {
            Ok(()) => {
                self.metrics
                    .push(CentsWithdrawal {
                        user_canister,
                        amount,
                    })
                    .await
                    .unwrap();
                Response::ok("done")
            }
            Err(e) => {
                self.dolr_treasury.rollback(&mut storage, amount).await?;
                Response::error(e.to_string(), 500u16)
            }
        }
    }

    async fn claim_gdollr(&mut self, user_canister: Principal, amount: Nat) -> Result<Response> {
        let on_chain_bal = self.backend.game_balance(user_canister).await?;
        if on_chain_bal.withdrawable >= amount {
            let res = self.redeem_gdollr(user_canister, amount).await;
            return res;
        }

        let effective_bal = self.effective_balance_info_inner(on_chain_bal).await?;
        if amount > effective_bal.withdrawable {
            return Response::error("not enough balance", 400);
        }

        self.settle_balance(user_canister).await?;

        self.redeem_gdollr(user_canister, amount).await
    }

    async fn claim_gdollr_v2(&mut self, user_canister: Principal, amount: Nat) -> Result<Response> {
        let on_chain_bal = self.backend.game_balance_v2(user_canister).await?;
        if on_chain_bal.withdrawable >= amount {
            let res = self.redeem_gdollr(user_canister, amount).await;
            return res;
        }

        let effective_bal = self.effective_balance_info_inner_v2(on_chain_bal).await?;
        if amount > effective_bal.withdrawable {
            return Response::error("not enough balance", 400);
        }

        self.settle_balance(user_canister).await?;

        self.redeem_gdollr(user_canister, amount).await
    }

    async fn effective_game_count(&mut self, user_canister: Principal) -> Result<u64> {
        let on_chain_count = self.backend.game_count(user_canister).await?;
        let off_chain_count = self.state_diffs().await?.len() + self.pending_games().await?.len();

        Ok(on_chain_count + off_chain_count as u64)
    }

    async fn effective_net_earnings(&mut self, user_canister: Principal) -> Result<Nat> {
        let on_chain_earnings = self.backend.net_earnings(user_canister).await?;
        let off_chain_earnings = self.off_chain_earning_delta().await?.clone();

        Ok(on_chain_earnings + off_chain_earnings)
    }
}

#[durable_object]
impl DurableObject for UserEphemeralState {
    fn new(state: State, env: Env) -> Self {
        console_error_panic_hook::set_once();

        let backend = StateBackend::new(&env).unwrap();

        Self {
            state,
            env,
            off_chain_balance_delta: StorageCell::new("off_chain_balance_delta", || {
                BigInt::from(0)
            }),
            off_chain_earning_delta: None,
            user_canister: None,
            state_diffs: None,
            pending_games: None,
            dolr_treasury: DolrTreasury::default(),
            backend,
            metrics: metrics(),
        }
    }

    async fn fetch(&mut self, req: Request) -> Result<Response> {
        let env = self.env.clone();
        let router = Router::with_data(self);

        router
            .get_async("/balance/:user_canister", |_req, ctx| async {
                let user_canister_raw = ctx.param("user_canister").unwrap();
                let Ok(user_canister) = Principal::from_text(user_canister_raw) else {
                    return Response::error("Invalid user_canister", 400);
                };

                let this = ctx.data;
                this.set_user_canister(user_canister).await?;
                let bal = this.effective_balance_info(user_canister).await?;
                Response::from_json(&bal)
            })
            .get_async("/balance_v2/:user_canister", |_req, ctx| async {
                let user_canister = parse_principal!(ctx, "user_canister");

                let this = ctx.data;
                this.set_user_canister(user_canister).await?;
                let bal = this.effective_balance_info_v2(user_canister).await?;
                Response::from_json(&bal)
            })
            .get_async("/earnings/:user_canister", |_req, ctx| async {
                let user_canister = parse_principal!(ctx, "user_canister");

                let this = ctx.data;
                this.set_user_canister(user_canister).await?;
                let earnings = this.effective_net_earnings(user_canister).await?;
                Response::ok(earnings.to_string())
            })
            .post_async("/decrement", |mut req, ctx| async move {
                let this = ctx.data;
                let decr_req: DecrementReq = req.json().await?;
                this.set_user_canister(decr_req.user_canister).await?;

                let bal = this.effective_balance(decr_req.user_canister).await?;
                if bal < GDOLLR_TO_E8S {
                    return Response::error("Not enough balance", 400);
                }
                let res = this.decrement(decr_req.token_root).await;
                if let Err(e) = res {
                    return Response::error(format!("failed to decrement: {e}"), 500);
                }

                Response::ok("done")
            })
            .post_async("/add_reward", |mut req, ctx| async move {
                let this = ctx.data;
                let reward_req: AddRewardReq = req.json().await?;

                this.set_user_canister(reward_req.user_canister).await?;
                this.add_state_diff(reward_req.state_diff).await?;

                Response::ok("done")
            })
            .post_async("/claim_gdollr", |mut req, ctx| async move {
                let this = ctx.data;
                let claim_req: ClaimGdollrReq = req.json().await?;

                this.set_user_canister(claim_req.user_canister).await?;

                this.claim_gdollr(claim_req.user_canister, claim_req.amount)
                    .await
            })
            .post_async("/claim_gdollr_v2", |mut req, ctx| async move {
                let this = ctx.data;
                let claim_req: ClaimGdollrReq = req.json().await?;

                this.set_user_canister(claim_req.user_canister).await?;

                this.claim_gdollr_v2(claim_req.user_canister, claim_req.amount)
                    .await
            })
            .get_async("/game_count/:user_canister", |_req, ctx| async move {
                let user_canister_raw = ctx.param("user_canister").unwrap();
                let Ok(user_canister) = Principal::from_text(user_canister_raw) else {
                    return Response::error("Invalid user_canister", 400);
                };

                let this = ctx.data;
                this.set_user_canister(user_canister).await?;
                let cnt = this.effective_game_count(user_canister).await?;

                Response::ok(cnt.to_string())
            })
            .get_async(
                "/uncommitted_games/:user_canister",
                |_req, ctx| async move {
                    let user_canister = parse_principal!(ctx, "user_canister");

                    let this = ctx.data;
                    this.set_user_canister(user_canister).await?;
                    let mut pending_games = this
                        .pending_games()
                        .await?
                        .iter()
                        .map(|p| UncommittedGameInfo::Pending { token_root: *p })
                        .collect::<Vec<_>>();
                    let completed_games =
                        this.state_diffs()
                            .await?
                            .iter()
                            .filter_map(|diff| match diff {
                                StateDiff::CompletedGame(g) => {
                                    Some(UncommittedGameInfo::Completed(g.clone()))
                                }
                                _ => None,
                            });
                    pending_games.extend(completed_games);

                    Response::from_json(&pending_games)
                },
            )
            .run(req, env)
            .await
    }

    async fn alarm(&mut self) -> Result<Response> {
        let Some(user_canister) = self.try_get_user_canister().await else {
            console_warn!("alarm set without user_canister set?!");
            return Response::ok("not ready");
        };

        if self.state_diffs().await?.is_empty() {
            console_warn!("alarm set without any updates?!");
            return Response::ok("not required");
        }

        self.settle_balance(user_canister).await?;

        Response::ok("done")
    }
}
