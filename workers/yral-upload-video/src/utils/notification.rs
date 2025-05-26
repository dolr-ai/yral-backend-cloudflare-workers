use std::fmt::Display;

use candid::Principal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::console_error;

const METADATA_SERVER_URL: &str = "https://yral-metadata.fly.dev";

pub struct NotificationClient {
    api_key: String,
}

impl NotificationClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn send_notification(&self, data: NotificationType, creator: Option<Principal>) {
        match creator {
            Some(creator_principal) => {
                let client = reqwest::Client::new();
                let url = format!(
                    "{}/notifications/{}/send",
                    METADATA_SERVER_URL,
                    creator_principal.to_text()
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
                        console_error!("Error sending notification request for video: {}", req_err);
                    }
                }
            }
            None => {
                console_error!("Creator principal not found for video, cannot send notification.");
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum NotificationType {
    VideoUploadSuccess,
    VideoUploadError,
}

impl Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::VideoUploadSuccess => {
                write!(f, "Your post was successfully uploaded. Tap here to view it")
            }
            NotificationType::VideoUploadError => write!(f, "Error uploading video"),
        }
    }
}



