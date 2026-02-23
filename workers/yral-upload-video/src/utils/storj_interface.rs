use reqwest::{multipart, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;

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

    pub fn get_upload_url(&self, video_id: &str, publisher_user_id: &str, is_nsfw: bool) -> String {
        format!(
            "{}/duplicate_raw/upload?publisher_user_id={}&video_id={}&is_nsfw={}",
            self.base_url, publisher_user_id, video_id, is_nsfw
        )
    }

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

        let part = multipart::Part::bytes(video_bytes)
            .file_name("video.mp4")
            .mime_str("video/mp4")?;
        let form = multipart::Form::new().part("file", part);

        let response = self.client.post(&url).multipart(form).send().await?;

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

        worker::console_log!("Finalize request URL: {}", url);
        worker::console_log!(
            "Finalize request body: {}",
            serde_json::to_string(&finalize_request).unwrap_or_default()
        );

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
