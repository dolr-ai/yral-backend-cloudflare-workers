use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

#[derive(Clone)]
pub struct StorjInterface {
    base_url: String,
    auth_token: String,
    client: Client,
}

impl StorjInterface {
    pub fn new(base_url: String, auth_token: String) -> Result<Self, Box<dyn Error>> {
        let client = Client::new();
        Ok(Self {
            base_url,
            auth_token,
            client,
        })
    }

    /// Downloads video from Cloudflare Stream
    pub async fn download_video_from_cf(&self, video_id: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let download_url = format!(
            "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/{}/downloads/default.mp4",
            video_id
        );

        let response = self.client.get(&download_url).send().await?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to download video from Cloudflare: {}",
                response.status()
            )
            .into());
        }

        let video_bytes = response.bytes().await?;
        Ok(video_bytes.to_vec())
    }

    /// Uploads video to Storj interface
    pub async fn upload_video(
        &self,
        video_id: &str,
        publisher_user_id: &str,
        is_nsfw: bool,
        video_bytes: Vec<u8>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn Error>> {
        let url = format!(
            "{}/duplicate_raw?publisher_user_id={}&video_id={}&is_nsfw={}",
            self.base_url, publisher_user_id, video_id, is_nsfw
        );

        let mut request = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.auth_token))
            .header("Content-Type", "application/octet-stream")
            .body(video_bytes);

        // Add metadata header if provided
        if let Some(meta) = metadata {
            let meta_json = serde_json::to_string(&meta)?;
            request = request.header("X-Metadata", meta_json);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Failed to upload video to Storj: {} - {}",
                status, error_body
            )
            .into());
        }

        Ok(())
    }

    /// Downloads from Cloudflare and uploads to Storj in one operation
    pub async fn duplicate_video_from_cf_to_storj(
        &self,
        video_id: &str,
        publisher_user_id: &str,
        is_nsfw: bool,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<(), Box<dyn Error>> {
        // Download from Cloudflare Stream
        let video_bytes = self.download_video_from_cf(video_id).await?;

        // Upload to Storj
        self.upload_video(video_id, publisher_user_id, is_nsfw, video_bytes, metadata)
            .await?;

        Ok(())
    }
}
