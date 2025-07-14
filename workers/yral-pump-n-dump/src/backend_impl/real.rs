use std::error::Error;

use candid::{Nat, Principal};
use worker::Result;
use yral_canisters_client::{
    individual_user_template::{BalanceInfo, PumpNDumpStateDiff, Result_},
    sns_ledger::{Account, TransferArg, TransferResult},
};

use crate::admin_cans::AdminCans;

use super::{GameBackendImpl, UserStateBackendImpl, WsBackendImpl};

fn to_worker_error(e: impl Error) -> worker::Error {
    worker::Error::RustError(e.to_string())
}

fn from_can_res(r: Result_) -> worker::Result<()> {
    match r {
        Result_::Ok => Ok(()),
        Result_::Err(e) => Err(worker::Error::RustError(e)),
    }
}

impl GameBackendImpl for AdminCans {
    async fn add_dollr_to_liquidity_pool(
        &self,
        user_canister: Principal,
        token_root: Principal,
        amount: Nat,
    ) -> Result<()> {
        let user = self.individual_user(user_canister).await;
        let res = user
            .add_dollr_to_liquidity_pool(token_root, amount)
            .await
            .map_err(to_worker_error)?;

        from_can_res(res)
    }
}

impl UserStateBackendImpl for AdminCans {
    async fn game_balance(&self, user_canister: Principal) -> Result<BalanceInfo> {
        let user = self.individual_user(user_canister).await;
        user.pd_balance_info().await.map_err(to_worker_error)
    }

    async fn game_balance_v2(&self, user_canister: Principal) -> Result<BalanceInfo> {
        let user = self.individual_user(user_canister).await;
        user.cents_token_balance_info()
            .await
            .map_err(to_worker_error)
    }

    async fn reconcile_user_state(
        &self,
        user_canister: Principal,
        completed_games: Vec<PumpNDumpStateDiff>,
    ) -> Result<()> {
        let user = self.individual_user(user_canister).await;
        let res = user
            .reconcile_user_state(completed_games)
            .await
            .map_err(to_worker_error)?;

        from_can_res(res)
    }

    async fn redeem_gdollr(&self, user_canister: Principal, amount: Nat) -> Result<()> {
        let user = self.individual_user(user_canister).await;
        let res = user.redeem_gdollr(amount).await.map_err(to_worker_error)?;

        from_can_res(res)
    }

    async fn game_count(&self, user_canister: Principal) -> Result<u64> {
        let user = self.individual_user(user_canister).await;

        user.played_game_count().await.map_err(to_worker_error)
    }

    async fn net_earnings(&self, user_canister: Principal) -> Result<Nat> {
        let user = self.individual_user(user_canister).await;

        user.net_earnings().await.map_err(to_worker_error)
    }

    async fn canister_controller(&self, user_canister: Principal) -> Result<Principal> {
        self.agent
            .canister_controller(user_canister)
            .await
            .map_err(to_worker_error)
    }

    async fn dolr_balance(&self, user_index: Principal) -> Result<Nat> {
        let ledger = self.dolr_ledger().await;

        ledger
            .icrc_1_balance_of(Account {
                owner: user_index,
                subaccount: None,
            })
            .await
            .map_err(to_worker_error)
    }

    async fn dolr_transfer(&self, to: Principal, amount: Nat) -> Result<()> {
        let ledger = self.dolr_ledger().await;

        let res = ledger
            .icrc_1_transfer(TransferArg {
                from_subaccount: None,
                to: Account {
                    owner: to,
                    subaccount: None,
                },
                amount,
                fee: None,
                memo: None,
                created_at_time: None,
            })
            .await
            .map_err(to_worker_error)?;

        if let TransferResult::Err(e) = res {
            return Err(worker::Error::RustError(format!("{e:?}")));
        }

        Ok(())
    }

    // async fn bet_on_hot_or_not_post(
    //     &self,
    //     user_canister: Principal,
    //     args: PlaceBetArg,
    // ) -> Result<std::result::Result<BettingStatus, BetOnCurrentlyViewingPostError>> {
    //     let user = self.individual_user(user_canister).await;
    //     let res = user
    //         .bet_on_currently_viewing_post_v_1(args)
    //         .await
    //         .map_err(to_worker_error)?;

    //     let status = match res {
    //         Result3::Ok(betting_status) => Ok(betting_status),
    //         Result3::Err(err) => Err(err),
    //     };

    //     Ok(status)
    // }
}

impl WsBackendImpl for AdminCans {
    async fn user_principal_to_user_canister(
        &self,
        user_principal: Principal,
    ) -> Result<Option<Principal>> {
        self.user_principal_to_user_canister(user_principal).await
    }

    async fn validate_token(
        &self,
        token_root: Principal,
        token_creator: Principal,
    ) -> Result<bool> {
        let user = self.individual_user(token_creator).await;
        let tokens = user
            .deployed_cdao_canisters()
            .await
            .map_err(to_worker_error)?;

        Ok(tokens.into_iter().any(|cans| cans.root == token_root))
    }
}
