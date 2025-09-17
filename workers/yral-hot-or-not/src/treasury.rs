use candid::{Nat, Principal};
use enum_dispatch::enum_dispatch;
use hon_worker_common::WorkerError;
use ic_agent::identity::Secp256k1Identity;
use worker::{console_log, Env};
use worker_utils::{
    environment::{env_kind, RunEnv},
    icp::agent_wrapper::AgentWrapper,
};
use yral_canisters_client::sns_ledger::{
    Account, SnsLedger, TransferArg, TransferError, TransferResult,
};

use crate::consts::CKBTC_LEDGER;

#[allow(unused)]
#[enum_dispatch]
pub(crate) trait CkBtcTreasury {
    async fn transfer_ckbtc(
        &self,
        to: Principal,
        amount: Nat,
        memo_text: Option<String>,
    ) -> Result<(), (u16, WorkerError)>;
}

pub struct NoOpCkBtcTreasury;

impl CkBtcTreasury for NoOpCkBtcTreasury {
    async fn transfer_ckbtc(
        &self,
        _to: Principal,
        _amount: Nat,
        _memo_text: Option<String>,
    ) -> Result<(), (u16, WorkerError)> {
        Ok(())
    }
}

#[allow(unused)]
pub struct AdminCkBtcTreasury(AgentWrapper);

impl AdminCkBtcTreasury {
    pub fn new(env: &Env) -> Result<Self, worker::Error> {
        let admin_pem = env.secret("BACKEND_ADMIN_KEY")?.to_string();
        let id = Secp256k1Identity::from_pem(admin_pem.as_bytes())
            .map_err(|e| worker::Error::RustError(e.to_string()))?;
        let agent = AgentWrapper::new(id);

        Ok(Self(agent))
    }
}

impl CkBtcTreasury for AdminCkBtcTreasury {
    async fn transfer_ckbtc(
        &self,
        to: Principal,
        amount: Nat,
        memo_text: Option<String>,
    ) -> Result<(), (u16, WorkerError)> {
        console_log!("ledger: {}; to: {}", CKBTC_LEDGER.to_text(), to.to_text());
        let ledger = SnsLedger(CKBTC_LEDGER, self.0.get().await);

        let memo = memo_text.unwrap_or_else(|| "Memo not specified".to_string());

        let res = ledger
            .icrc_1_transfer(TransferArg {
                to: Account {
                    owner: to,
                    subaccount: None,
                },
                fee: None,
                memo: Some(Vec::from(memo).into()),
                from_subaccount: None,
                created_at_time: None,
                amount: amount.clone(),
            })
            .await
            .map_err(|e| (500, WorkerError::Internal(e.to_string())))?;
        match res {
            TransferResult::Err(TransferError::InsufficientFunds { .. }) => {
                return Err((500, WorkerError::TreasuryOutOfFunds))
            }
            TransferResult::Err(e) => {
                return Err((500, WorkerError::Internal(format!("{e:?}"))));
            }
            TransferResult::Ok(_) => (),
        }

        Ok(())
    }
}

#[enum_dispatch(CkBtcTreasury)]
pub enum CkBtcTreasuryImpl {
    Mock(NoOpCkBtcTreasury),
    Real(AdminCkBtcTreasury),
}

impl CkBtcTreasuryImpl {
    pub fn new(env: &Env) -> Result<Self, worker::Error> {
        let this = match env_kind() {
            RunEnv::Remote => Self::Real(AdminCkBtcTreasury::new(env)?),
            _ => Self::Mock(NoOpCkBtcTreasury),
        };

        Ok(this)
    }
}
