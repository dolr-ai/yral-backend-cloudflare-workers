use std::error::Error;

use candid::Principal;
use ic_agent::{identity::DelegatedIdentity, Agent};
use serde::{Deserialize, Serialize};
use worker::{console_error, console_log};

use crate::utils::{
    cloudflare_stream::{self, CloudflareStream},
    events::EventService,
    individual_user_canister::{
        PostDetailsFromFrontend, Result2, Service as IndividualUserCanisterService,
    },
    types::DelegatedIdentityWire,
};
#[derive(Serialize, Deserialize)]
pub struct UploadVideoToCanisterResult {
    pub cans_id: Principal,
    pub post_id: u64,
}
pub async fn upload_video_to_canister(
    cloudflare_stream: &CloudflareStream,
    events: &EventService,
    video_uid: String,
    ic_agent: &Agent,
    post_details: PostDetailsFromFrontend,
    country: Option<String>,
    city: Option<String>,
) -> Result<UploadVideoToCanisterResult, Box<dyn Error>> {
    let yral_metadata_client = yral_metadata_client::MetadataClient::default();

    console_log!("user principal id {}", ic_agent.get_principal()?);

    let user_details = yral_metadata_client
        .get_user_metadata(ic_agent.get_principal()?)
        .await?
        .ok_or::<Box<dyn Error>>("user canister not found".into())?;

    console_log!("user canister id {}", user_details.user_canister_id);

    let individual_user_service =
        IndividualUserCanisterService(user_details.user_canister_id, &ic_agent);

    match upload_video_to_canister_and_mark_video_for_download(
        cloudflare_stream,
        &video_uid,
        &individual_user_service,
        post_details.clone(),
    )
    .await
    {
        Ok(post_id) => {
            console_log!("video upload to canister successful");

            let _ = events
                .send_video_upload_successful_event(
                    video_uid,
                    post_details.hashtags.len(),
                    post_details.is_nsfw,
                    post_details.creator_consent_for_inclusion_in_hot_or_not,
                    post_id,
                    ic_agent.get_principal()?,
                    user_details.user_canister_id,
                    user_details.user_name,
                    country,
                    city,
                )
                .await
                .inspect_err(|e| {
                    console_error!(
                        "Error sending video successful event. Error {}",
                        e.to_string()
                    )
                });

            Ok(UploadVideoToCanisterResult {
                cans_id: user_details.user_canister_id,
                post_id,
            })
        }
        Err(e) => {
            console_error!(
                "video upload to canister unsuccessful.Error {}",
                e.to_string()
            );
            let _ = events
                .send_video_event_unsuccessful(
                    e.to_string(),
                    post_details.hashtags.len(),
                    post_details.is_nsfw,
                    post_details.creator_consent_for_inclusion_in_hot_or_not,
                    ic_agent.get_principal()?,
                    user_details.user_name,
                    user_details.user_canister_id,
                )
                .await
                .inspect_err(|e| {
                    console_error!(
                        "Error sending video unsuccessful event. Error {}",
                        e.to_string()
                    )
                });

            Err(e)
        }
    }
}

async fn upload_video_to_canister_and_mark_video_for_download(
    cloudflare_stream: &CloudflareStream,
    video_uid: &str,
    individual_user_canister_service: &IndividualUserCanisterService<'_>,
    post_details: PostDetailsFromFrontend,
) -> Result<u64, Box<dyn Error>> {
    let result =
        upload_video_to_canister_inner(individual_user_canister_service, post_details).await;

    cloudflare_stream
        .mark_video_as_downloadable(video_uid)
        .await?;

    result
}

async fn upload_video_to_canister_inner(
    individual_user_canister: &IndividualUserCanisterService<'_>,
    post_details: PostDetailsFromFrontend,
) -> Result<u64, Box<dyn Error>> {
    let result = individual_user_canister.add_post_v_2(post_details).await?;
    match result {
        Result2::Ok(post_id) => Ok(post_id),
        Result2::Err(err) => Err(err.into()),
    }
}
