use std::error::Error;

use candid::Principal;
use yral_canisters_client::ic::USER_INFO_SERVICE_ID;

pub struct RedisRestClient {
    reqwest_client: reqwest::Client,
    base_url: reqwest::Url,
    auth_token: String,
}

impl RedisRestClient {
    pub fn new(base_url: String, auth_token: String) -> Result<Self, Box<dyn Error>> {
        let reqwest_client = reqwest::Client::new();
        let base_url = reqwest::Url::parse(&base_url)?;

        Ok(Self {
            reqwest_client,
            base_url,
            auth_token,
        })
    }

    pub async fn set_value(
        &self,
        old_canister: Principal,
        post_id: u64,
        new_post_id: String,
    ) -> Result<(), Box<dyn Error>> {
        let path = format!("set/{old_canister}:{post_id}/{USER_INFO_SERVICE_ID}:{new_post_id}");

        let response = self
            .reqwest_client
            .post(self.base_url.join(&path).unwrap())
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error = response.text().await?;
            Err(format!("error setting value in redis. Error {status} {error}",).into())
        }
    }
}
