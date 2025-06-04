use hon_worker_common::ReferralItem;
use serde::{Deserialize, Serialize};
use worker::Result;
use worker_utils::storage::SafeStorage;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ReferralInner {
    pub referral_history: Vec<ReferralItem>,
    pub referred_by: Option<ReferralItem>,
}

#[derive(Default, Clone)]
pub struct ReferralStore(Option<ReferralInner>);

impl ReferralStore {
    async fn get_or_init(&mut self, storage: &SafeStorage) -> Result<&mut ReferralInner> {
        if self.0.is_some() {
            return Ok(self.0.as_mut().unwrap());
        }

        let referral = storage.get("referral").await?.unwrap_or_default();
        self.0 = Some(referral);
        Ok(self.0.as_mut().unwrap())
    }

    pub async fn referral_history(
        &mut self,
        storage: &mut SafeStorage,
    ) -> Result<&mut Vec<ReferralItem>> {
        let referral = self.get_or_init(storage).await?;
        Ok(&mut referral.referral_history)
    }

    pub async fn referred_by(
        &mut self,
        storage: &mut SafeStorage,
    ) -> Result<&mut Option<ReferralItem>> {
        let referral = self.get_or_init(storage).await?;
        Ok(&mut referral.referred_by)
    }

    pub async fn add_referral_history(
        &mut self,
        storage: &mut SafeStorage,
        referral_item: ReferralItem,
    ) -> Result<()> {
        let referral = self.get_or_init(storage).await?;
        referral.referral_history.push(referral_item);

        storage.put("referral", referral).await?;

        Ok(())
    }

    pub async fn add_referred_by(
        &mut self,
        storage: &mut SafeStorage,
        referral_item: ReferralItem,
    ) -> Result<()> {
        let referral = self.get_or_init(storage).await?;

        if referral.referred_by.is_some() || referral.referral_history.len() > 0 {
            return Err(worker::Error::RustError("referral already exists".into()));
        }

        referral.referred_by = Some(referral_item);

        storage.put("referral", referral).await?;

        Ok(())
    }
}
