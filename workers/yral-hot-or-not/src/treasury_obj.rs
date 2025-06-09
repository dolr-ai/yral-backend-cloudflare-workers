use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use worker::{console_debug, Date, Result};
use worker_utils::storage::SafeStorage;

use crate::consts::{CKBTC_TREASURY_STORAGE_KEY, MAXIMUM_CKBTC_TREASURY_PER_DAY_PER_USER};

#[derive(Serialize, Deserialize, Clone)]
struct CkBtcTreasuryInner {
    amount: BigUint,
    last_reset_epoch: u64,
}

impl Default for CkBtcTreasuryInner {
    fn default() -> Self {
        Self {
            amount: BigUint::from(MAXIMUM_CKBTC_TREASURY_PER_DAY_PER_USER),
            last_reset_epoch: Date::now().as_millis(),
        }
    }
}

#[derive(Default, Clone)]
pub struct CkBtcTreasuryStore(Option<CkBtcTreasuryInner>);

impl CkBtcTreasuryStore {
    async fn get_or_init(&mut self, storage: &SafeStorage) -> Result<&mut CkBtcTreasuryInner> {
        if self.0.is_some() {
            return Ok(self.0.as_mut().unwrap());
        }

        let dolr_treasury_limit = storage
            .get(CKBTC_TREASURY_STORAGE_KEY)
            .await?
            .unwrap_or_default();
        self.0 = Some(dolr_treasury_limit);
        Ok(self.0.as_mut().unwrap())
    }

    async fn treasury(&mut self, storage: &mut SafeStorage) -> Result<&mut CkBtcTreasuryInner> {
        let treasury = self.get_or_init(storage).await?;
        if Date::now().as_millis() - (24 * 3600 * 1000) >= treasury.last_reset_epoch {
            *treasury = CkBtcTreasuryInner::default();
            storage.put(CKBTC_TREASURY_STORAGE_KEY, treasury).await?;
        };

        Ok(treasury)
    }

    pub async fn try_consume(&mut self, storage: &mut SafeStorage, amount: BigUint) -> Result<()> {
        let treasury = self.treasury(storage).await?;
        console_debug!("treasury amount: {}", treasury.amount.clone());
        if treasury.amount.clone() < amount {
            return Err(worker::Error::RustError("daily limit reached".into()));
        }
        treasury.amount -= amount;
        storage.put(CKBTC_TREASURY_STORAGE_KEY, treasury).await?;

        Ok(())
    }

    pub async fn rollback(&mut self, storage: &mut SafeStorage, amount: BigUint) -> Result<()> {
        let treasury = self.treasury(storage).await?;
        treasury.amount =
            (treasury.amount.clone() + amount).min(MAXIMUM_CKBTC_TREASURY_PER_DAY_PER_USER.into());
        storage.put(CKBTC_TREASURY_STORAGE_KEY, treasury).await?;

        Ok(())
    }

    // pub async fn amount(&mut self, storage: &mut SafeStorage) -> Result<BigUint> {
    //     self.treasury(storage).await.map(|v| v.amount.clone())
    // }
}
