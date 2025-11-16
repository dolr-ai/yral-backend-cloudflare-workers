use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use worker::console_log;

#[derive(Clone)]
pub struct StorjInterface {
    base_url: String,
    client: Client,
}

#[derive(Serialize, Deserialize)]
pub struct FinalizeRequest {
    pub metadata: HashMap<String, String>,
}

impl StorjInterface {
    pub fn new(base_url: String) -> Result<Self, Box<dyn Error>> {
        let client = Client::new();
        Ok(Self { base_url, client })
    }

    pub async fn download_video_from_cf(&self, video_id: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let download_url = format!(
            "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/{}/downloads/default.mp4",
            video_id
        );

        let max_retries = 5;
        let mut delay_ms = 4000;

        for attempt in 1..=max_retries {
            console_log!(
                "Attempting to download video from CF (attempt {}/{})",
                attempt,
                max_retries
            );

            let response = self.client.get(&download_url).send().await?;

            if response.status().is_success() {
                let video_bytes = response.bytes().await?;
                console_log!(
                    "Successfully downloaded video from CF ({} bytes)",
                    video_bytes.len()
                );
                return Ok(video_bytes.to_vec());
            }

            if attempt < max_retries {
                console_log!(
                    "Download failed with status {}, waiting {}ms before retry",
                    response.status(),
                    delay_ms
                );
                worker::Delay::from(std::time::Duration::from_millis(delay_ms)).await;
                delay_ms *= 2;
            } else {
                return Err(format!(
                    "Failed to download video from Cloudflare after {} attempts: {}",
                    max_retries,
                    response.status()
                )
                .into());
            }
        }

        Err("Failed to download video from Cloudflare".into())
    }

    pub async fn upload_pending(
        &self,
        video_id: &str,
        publisher_user_id: &str,
        is_nsfw: bool,
        video_bytes: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/duplicate_raw/upload?publisher_user_id={}&video_id={}&is_nsfw={}",
            self.base_url, publisher_user_id, video_id, is_nsfw
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/octet-stream")
            .body(video_bytes)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Failed to upload pending video to Storj: {} - {}",
                status, error_body
            )
            .into());
        }

        Ok(())
    }

    pub async fn finalize_upload(
        &self,
        video_id: &str,
        publisher_user_id: &str,
        is_nsfw: bool,
        metadata: HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/duplicate_raw/finalize?publisher_user_id={}&video_id={}&is_nsfw={}",
            self.base_url, publisher_user_id, video_id, is_nsfw
        );

        let finalize_request = FinalizeRequest { metadata };

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&finalize_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Failed to finalize video upload to Storj: {} - {}",
                status, error_body
            )
            .into());
        }

        Ok(())
    }

    pub async fn duplicate_video_from_cf_to_storj(
        &self,
        video_id: &str,
        publisher_user_id: &str,
        is_nsfw: bool,
        metadata: HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        let video_bytes = self.download_video_from_cf(video_id).await?;

        self.upload_pending(video_id, publisher_user_id, is_nsfw, video_bytes)
            .await?;

        self.finalize_upload(video_id, publisher_user_id, is_nsfw, metadata)
            .await?;

        Ok(())
    }
}
