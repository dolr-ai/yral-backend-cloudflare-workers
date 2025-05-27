use std::fmt::Display;

use candid::Principal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::console_error;

use crate::consts::{REFERRAL_REWARD_REFEREE_SATS, REFERRAL_REWARD_REFERRER_SATS};

const METADATA_SERVER_URL: &str = "https://yral-metadata.fly.dev";

pub struct NotificationClient {
    api_key: String,
}

impl NotificationClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn send_notification(
        &self,
        data: NotificationType,
        user_principal: Option<Principal>,
    ) {
        match user_principal {
            Some(user_principal) => {
                let client = reqwest::Client::new();
                let url = format!(
                    "{}/notifications/{}/send",
                    METADATA_SERVER_URL,
                    user_principal.to_text()
                );

                let res = client
                    .post(&url)
                    .bearer_auth(&self.api_key)
                    .json(&json!({ "data": {
                        "title": data.to_string(),
                        "body": data.to_string(),
                    }}))
                    .send()
                    .await;

                match res {
                    Ok(response) => {
                        if response.status().is_success() {
                        } else {
                            if let Ok(body) = response.text().await {
                                console_error!("Response body: {}", body);
                            }
                        }
                    }
                    Err(req_err) => {
                        console_error!("Error sending notification request for : {}", req_err);
                    }
                }
            }
            None => {
                console_error!("User principal not found, cannot send notification.");
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum NotificationType {
    ReferrerReferralReward { referee_principal: Principal },
    RefereeReferralReward,
}

impl Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::ReferrerReferralReward { referee_principal } => {
                write!(
                    f,
                    "You have received a referral reward of {} SATS. User Joined {}",
                    REFERRAL_REWARD_REFERRER_SATS,
                    referee_principal.to_text()
                )
            }
            NotificationType::RefereeReferralReward => {
                write!(
                    f,
                    "You have received a referral reward of {} SATS",
                    REFERRAL_REWARD_REFEREE_SATS
                )
            }
        }
    }
}
