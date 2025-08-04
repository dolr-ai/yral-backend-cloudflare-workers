use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct YralBalanceInfo {
    #[serde_as(as = "DisplayFromStr")]
    pub balance: BigUint,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
pub struct YralBalanceUpdateRequest {
    pub previous_balance: BigUint,
    #[serde_as(as = "DisplayFromStr")]
    pub delta: BigInt,
}
