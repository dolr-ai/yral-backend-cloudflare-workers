use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use worker::{Date, Result};

use crate::storage::{SafeStorage, StorageCell};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct CumulativeInner<const MAX_VAL: u64> {
    amount: BigUint,
    last_reset_epoch: u64,
}

impl<const MAX_VAL: u64> Default for CumulativeInner<MAX_VAL> {
    fn default() -> Self {
        Self {
            amount: BigUint::from(MAX_VAL),
            last_reset_epoch: Date::now().as_millis(),
        }
    }
}

pub struct DailyCumulativeLimit<const MAX_VAL: u64>(StorageCell<CumulativeInner<MAX_VAL>>);

impl<const MAX_VAL: u64> DailyCumulativeLimit<MAX_VAL> {
    pub fn new(key: impl AsRef<str>) -> Self {
        Self(StorageCell::new(key, CumulativeInner::<MAX_VAL>::default))
    }

    pub async fn try_consume(&mut self, storage: &mut SafeStorage, amount: BigUint) -> Result<()> {
        let mut err = None::<worker::Error>;
        self.0
            .update(storage, |inner| {
                if Date::now().as_millis() - (24 * 3600 * 1000) >= inner.last_reset_epoch {
                    *inner = CumulativeInner::<MAX_VAL>::default();
                }
                if inner.amount < amount {
                    err = Some(worker::Error::RustError("daily limit reached".into()));
                    return;
                }
                inner.amount -= amount;
            })
            .await?;

        if let Some(e) = err {
            return Err(e);
        }
        Ok(())
    }

    pub async fn rollback(&mut self, storage: &mut SafeStorage, amount: BigUint) -> Result<()> {
        self.0
            .update(storage, |inner| {
                inner.amount = (inner.amount.clone() + amount).min(MAX_VAL.into());
            })
            .await
    }
}
