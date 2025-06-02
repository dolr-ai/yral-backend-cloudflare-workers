use std::fmt::Display;

use candid::Principal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use worker::console_error;
use yral_metadata_types::{
    AndroidConfig, AndroidNotification, ApnsConfig, ApnsFcmOptions, NotificationPayload,
    SendNotificationReq, WebpushConfig, WebpushFcmOptions,
};

use crate::server_impl::upload_video_to_canister::UploadVideoToCanisterResult;

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
                    .json(&SendNotificationReq{
                        notification: Some(NotificationPayload{
                            title: Some(data.to_string()),
                            body: Some(data.to_string()),
                            image: Some("https://yral.com/img/yral/android-chrome-384x384.png".to_string()),
                        }),
                        android: Some(AndroidConfig{
                            notification: Some(AndroidNotification{
                                icon: Some("https://yral.com/img/yral/android-chrome-384x384.png".to_string()),
                                click_action: if let NotificationType::VideoUploadSuccess(ref post_id) = data {
                                    Some(format!("https://yral.com/{}/{}", post_id.cans_id.to_text(), post_id.post_id))
                                } else {None},
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        webpush: Some(WebpushConfig{
                            fcm_options: if let NotificationType::VideoUploadSuccess(ref post_id) = data {
                                Some(WebpushFcmOptions{
                                    link: Some(format!("https://yral.com/{}/{}", post_id.cans_id.to_text(), post_id.post_id)),
                                    ..Default::default()
                                })
                            } else {None},
                            ..Default::default()
                        }),
                        apns: Some(ApnsConfig{
                            fcm_options: Some(ApnsFcmOptions{
                                image: Some("https://yral.com/img/yral/android-chrome-384x384.png".to_string()),
                                ..Default::default()
                            }),
                            payload: if let NotificationType::VideoUploadSuccess(post_id) = data {
                                Some(json!({
                                    "url": format!("https://yral.com/{}/{}", post_id.cans_id.to_text(), post_id.post_id)
                                }))
                            } else {None},
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
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
    VideoUploadSuccess(UploadVideoToCanisterResult),
    VideoUploadError,
}

impl Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::VideoUploadSuccess(_) => {
                write!(
                    f,
                    "Your post was successfully uploaded. Tap here to view it"
                )
            }
            NotificationType::VideoUploadError => write!(f, "Error uploading video"),
        }
    }
}
