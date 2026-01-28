mod treasury;

use std::{cell::RefCell, collections::HashSet};

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
use yral_canisters_client::individual_user_template::{BalanceInfo, PumpNDumpStateDiff};
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
    off_chain_balance_delta: RefCell<StorageCell<BigInt>>,
    // effective earnings = on_chain_earnings + off_chain_earnings
    off_chain_earning_delta: RefCell<Option<Nat>>,
    user_canister: RefCell<Option<Principal>>,
    state_diffs: RefCell<Option<Vec<StateDiff>>>,
    pending_games: RefCell<Option<HashSet<Principal>>>,
    backend: StateBackend,
    dolr_treasury: RefCell<DolrTreasury>,
    metrics: CfMetricTx,
}

// SAFETY: RefCell borrows held across await points are safe in Cloudflare Workers
// because Workers run in a single-threaded JavaScript runtime with no concurrent access.
// The RefCell interior mutability pattern is required due to Worker 0.7.4 API changes
// that mandate `&self` instead of `&mut self` for DurableObject trait methods.
#[allow(clippy::await_holding_refcell_ref)]
impl UserEphemeralState {
    fn storage(&self) -> SafeStorage {
        self.state.storage().into()
    }

    async fn set_user_canister(&self, user_canister: Principal) -> Result<()> {
        if self.user_canister.borrow().is_some() {
            return Ok(());
        }

        *self.user_canister.borrow_mut() = Some(user_canister);
        self.storage().put("user_canister", &user_canister).await?;

        Ok(())
    }

    async fn try_get_user_canister(&self) -> Option<Principal> {
        if let Some(user_canister) = *self.user_canister.borrow() {
            return Some(user_canister);
        }

        let user_canister = self.storage().get("user_canister").await.ok()??;
        *self.user_canister.borrow_mut() = Some(user_canister);

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

    async fn ensure_off_chain_earning_delta_loaded(&self) -> Result<()> {
        if self.off_chain_earning_delta.borrow().is_some() {
            return Ok(());
        }

        let off_chain_earning_delta = self
            .storage()
            .get::<Nat>("off_chain_earning_delta")
            .await?
            .unwrap_or_default();
        *self.off_chain_earning_delta.borrow_mut() = Some(off_chain_earning_delta);
        Ok(())
    }

    async fn ensure_pending_games_loaded(&self) -> Result<()> {
        if self.pending_games.borrow().is_some() {
            return Ok(());
        }

        let pending_games = self
            .storage()
            .list_with_prefix("pending-game-")
            .await
            .map(|v| v.map(|v| v.1))
            .collect::<Result<_>>()?;

        *self.pending_games.borrow_mut() = Some(pending_games);
        Ok(())
    }

    async fn ensure_state_diffs_loaded(&self) -> Result<()> {
        if self.state_diffs.borrow().is_some() {
            return Ok(());
        }

        let state_diffs = self
            .storage()
            .list_with_prefix("state-diff-")
            .await
            .map(|v| v.map(|v| v.1))
            .collect::<Result<_>>()?;

        *self.state_diffs.borrow_mut() = Some(state_diffs);
        Ok(())
    }

    async fn effective_balance_inner(&self, on_chain_balance: Nat) -> Result<Nat> {
        let mut effective_balance = on_chain_balance;
        let off_chain_delta = self
            .off_chain_balance_delta
            .borrow_mut()
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

    async fn effective_balance(&self, user_canister: Principal) -> Result<Nat> {
        let on_chain_balance = self.backend.game_balance(user_canister).await?;

        self.effective_balance_inner(on_chain_balance.balance).await
    }

    async fn effective_balance_info_inner(&self, mut bal_info: BalanceInfo) -> Result<BalanceInfo> {
        bal_info.balance = self
            .effective_balance_inner(bal_info.balance.clone())
            .await?;

        bal_info.withdrawable = if bal_info.net_airdrop_reward > bal_info.balance {
            0u32.into()
        } else {
            let bal = bal_info.balance.clone() - bal_info.net_airdrop_reward.clone();
            let treasury = self
                .dolr_treasury
                .borrow_mut()
                .amount(&mut self.storage())
                .await?;
            bal.min(treasury)
        };

        Ok(bal_info)
    }

    async fn effective_balance_info_v2(
        &self,
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
        &self,
        mut bal_info: BalanceInfo,
    ) -> Result<BalanceInfo> {
        bal_info.balance = self
            .effective_balance_inner(bal_info.balance.clone())
            .await?;

        let treasury = self
            .dolr_treasury
            .borrow_mut()
            .amount(&mut self.storage())
            .await?;
        bal_info.withdrawable = bal_info.withdrawable.min(treasury);

        Ok(bal_info)
    }

    async fn effective_balance_info(
        &self,
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

    async fn decrement(&self, pending_game_root: Principal) -> Result<()> {
        let mut storage = self.storage();
        self.off_chain_balance_delta
            .borrow_mut()
            .update(&mut storage, |delta| *delta -= GDOLLR_TO_E8S)
            .await?;

        self.ensure_pending_games_loaded().await?;
        let inserted = self
            .pending_games
            .borrow_mut()
            .as_mut()
            .unwrap()
            .insert(pending_game_root);
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

    async fn add_state_diff_inner(&self, state_diff: StateDiff) -> Result<()> {
        let reward = state_diff.reward();
        let mut storage = self.storage();
        self.off_chain_balance_delta
            .borrow_mut()
            .update(&mut storage, |delta| *delta += BigInt::from(reward.clone()))
            .await?;

        self.ensure_off_chain_earning_delta_loaded().await?;
        *self.off_chain_earning_delta.borrow_mut().as_mut().unwrap() += reward;
        storage
            .put(
                "off_chain_earning_delta",
                self.off_chain_earning_delta.borrow().as_ref().unwrap(),
            )
            .await?;

        self.ensure_state_diffs_loaded().await?;
        let next_idx = {
            let mut state_diffs = self.state_diffs.borrow_mut();
            let state_diffs = state_diffs.as_mut().unwrap();
            state_diffs.push(state_diff.clone());
            state_diffs.len() - 1
        };

        if let StateDiff::CompletedGame(ginfo) = &state_diff {
            self.ensure_pending_games_loaded().await?;
            self.pending_games
                .borrow_mut()
                .as_mut()
                .unwrap()
                .remove(&ginfo.token_root);
            storage
                .delete(&format!("pending-game-{}", ginfo.token_root))
                .await?;
        }

        storage
            .put(&format!("state-diff-{next_idx}"), &state_diff)
            .await?;

        Ok(())
    }

    async fn add_state_diff(&self, state_diff: StateDiff) -> Result<()> {
        self.add_state_diff_inner(state_diff).await?;
        self.queue_settle_balance().await?;

        Ok(())
    }

    async fn settle_balance(&self, user_canister: Principal) -> Result<()> {
        let mut storage = self.storage();
        let to_settle = self
            .off_chain_balance_delta
            .borrow_mut()
            .read(&storage)
            .await?
            .clone();

        self.ensure_off_chain_earning_delta_loaded().await?;
        let earnings = self
            .off_chain_earning_delta
            .borrow()
            .as_ref()
            .unwrap()
            .clone();
        *self.off_chain_earning_delta.borrow_mut() = Some(0u32.into());
        storage.delete("off_chain_earning_delta").await?;

        self.ensure_state_diffs_loaded().await?;
        let state_diffs = std::mem::take(self.state_diffs.borrow_mut().as_mut().unwrap());
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
            .borrow_mut()
            .update(&mut storage, |delta| *delta += delta_delta)
            .await?;

        let res = self
            .backend
            .reconcile_user_state(user_canister, state_diffs_conv)
            .await;

        if let Err(e) = res {
            self.off_chain_balance_delta
                .borrow_mut()
                .set(&mut storage, to_settle)
                .await?;
            *self.state_diffs.borrow_mut() = Some(state_diffs.clone());
            *self.off_chain_earning_delta.borrow_mut() = Some(earnings.clone());

            storage.put("off_chain_earning_delta", &earnings).await?;

            for (i, state_diff) in state_diffs.into_iter().enumerate() {
                storage.put(&format!("state-diff-{i}"), &state_diff).await?;
            }

            return Err(e);
        }

        Ok(())
    }

    async fn check_user_index_balance(
        &self,
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

    async fn redeem_gdollr(&self, user_canister: Principal, amount: Nat) -> Result<Response> {
        let mut storage = self.storage();

        self.check_user_index_balance(user_canister, amount.clone())
            .await?;
        self.dolr_treasury
            .borrow_mut()
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
                self.dolr_treasury
                    .borrow_mut()
                    .rollback(&mut storage, amount)
                    .await?;
                Response::error(e.to_string(), 500u16)
            }
        }
    }

    async fn claim_gdollr(&self, user_canister: Principal, amount: Nat) -> Result<Response> {
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

    async fn claim_gdollr_v2(&self, user_canister: Principal, amount: Nat) -> Result<Response> {
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

    async fn effective_game_count(&self, user_canister: Principal) -> Result<u64> {
        let on_chain_count = self.backend.game_count(user_canister).await?;
        self.ensure_state_diffs_loaded().await?;
        self.ensure_pending_games_loaded().await?;
        let off_chain_count = self.state_diffs.borrow().as_ref().unwrap().len()
            + self.pending_games.borrow().as_ref().unwrap().len();

        Ok(on_chain_count + off_chain_count as u64)
    }

    async fn effective_net_earnings(&self, user_canister: Principal) -> Result<Nat> {
        let on_chain_earnings = self.backend.net_earnings(user_canister).await?;
        self.ensure_off_chain_earning_delta_loaded().await?;
        let off_chain_earnings = self
            .off_chain_earning_delta
            .borrow()
            .as_ref()
            .unwrap()
            .clone();

        Ok(on_chain_earnings + off_chain_earnings)
    }
}

// SAFETY: RefCell borrows held across await points are safe in Cloudflare Workers
// because Workers run in a single-threaded JavaScript runtime with no concurrent access.
// The RefCell interior mutability pattern is required due to Worker 0.7.4 API changes
// that mandate `&self` instead of `&mut self` for DurableObject trait methods.
#[allow(clippy::await_holding_refcell_ref)]
impl DurableObject for UserEphemeralState {
    fn new(state: State, env: Env) -> Self {
        console_error_panic_hook::set_once();

        let backend = StateBackend::new(&env).unwrap();

        Self {
            state,
            env,
            off_chain_balance_delta: RefCell::new(StorageCell::new(
                "off_chain_balance_delta",
                || BigInt::from(0),
            )),
            off_chain_earning_delta: RefCell::new(None),
            user_canister: RefCell::new(None),
            state_diffs: RefCell::new(None),
            pending_games: RefCell::new(None),
            dolr_treasury: RefCell::new(DolrTreasury::default()),
            backend,
            metrics: metrics(),
        }
    }

    async fn fetch(&self, req: Request) -> Result<Response> {
        let env = self.env.clone();
        let router = Router::with_data(self);

        router
            .get_async("/balance/:user_canister", |_req, ctx| async move {
                let user_canister_raw = ctx.param("user_canister").unwrap();
                let Ok(user_canister) = Principal::from_text(user_canister_raw) else {
                    return Response::error("Invalid user_canister", 400);
                };

                let this = ctx.data;
                this.set_user_canister(user_canister).await?;
                let bal = this.effective_balance_info(user_canister).await?;
                Response::from_json(&bal)
            })
            .get_async("/balance_v2/:user_canister", |_req, ctx| async move {
                let user_canister = parse_principal!(ctx, "user_canister");

                let this = ctx.data;
                this.set_user_canister(user_canister).await?;
                let bal = this.effective_balance_info_v2(user_canister).await?;
                Response::from_json(&bal)
            })
            .get_async("/earnings/:user_canister", |_req, ctx| async move {
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
                    this.ensure_pending_games_loaded().await?;
                    let mut pending_games = this
                        .pending_games
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|p| UncommittedGameInfo::Pending { token_root: *p })
                        .collect::<Vec<_>>();
                    this.ensure_state_diffs_loaded().await?;
                    let state_diffs_ref = this.state_diffs.borrow();
                    let completed_games =
                        state_diffs_ref
                            .as_ref()
                            .unwrap()
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

    async fn alarm(&self) -> Result<Response> {
        let Some(user_canister) = self.try_get_user_canister().await else {
            console_warn!("alarm set without user_canister set?!");
            return Response::ok("not ready");
        };

        self.ensure_state_diffs_loaded().await?;
        if self.state_diffs.borrow().as_ref().unwrap().is_empty() {
            console_warn!("alarm set without any updates?!");
            return Response::ok("not required");
        }

        self.settle_balance(user_canister).await?;

        Response::ok("done")
    }
}
