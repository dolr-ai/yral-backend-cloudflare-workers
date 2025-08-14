use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug, Error)]
pub enum WorkerError {
    #[error("internal error: {0}")]
    Internal(String),
    #[error("user does not have sufficient balance")]
    InsufficientFunds,
    #[error("conflict while updating balance, retry")]
    BalanceTransactionConflict { new_balance: BigUint },
    #[error("yral credit limit reached")]
    YralCreditLimitReached,
    #[error("yral deduct limit reached")]
    YralDeductLimitReached,
}
