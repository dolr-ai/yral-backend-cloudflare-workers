use std::error::Error;

use candid::Principal;
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker::{console_error, console_log};
use yral_canisters_client::{
    ic::USER_POST_SERVICE_ID,
    individual_user_template::{
        IndividualUserTemplate as IndividualUserCanisterService, PostDetailsFromFrontend,
        Result1 as AddPostResult,
    },
    local::USER_INFO_SERVICE_ID,
    user_info_service::{Result1, UserInfoService},
    user_post_service::{
        PostDetailsFromFrontend as PostServicePostDetailsFromFrontend, Result_, UserPostService,
    },
};

use crate::utils::{cloudflare_stream::CloudflareStream, events::EventService};
#[derive(Serialize, Deserialize)]
pub struct UploadVideoToCanisterResult {
    pub cans_id: Principal,
    pub post_id: u64,
}

pub async fn upload_video_to_canister_impl(
    user_ic_agent: &Agent,
    admin_ic_agent: &Agent,
    post_details: PostDetailsFromFrontend,
) -> Result<String, Box<dyn Error>> {
    let yral_metadata_client = yral_metadata_client::MetadataClient::default();

    let user_details_res = yral_metadata_client
        .get_user_metadata_v2(user_ic_agent.get_principal()?.to_string())
        .await?;

    let user_details = user_details_res.ok_or("User details not found")?;

    if user_details.user_canister_id != USER_INFO_SERVICE_ID {
        let individual_user_service =
            IndividualUserCanisterService(user_details.user_canister_id, user_ic_agent);

        let post_id =
            upload_video_to_individual_canister(&individual_user_service, post_details).await?;
        Ok(post_id.to_string())
    } else {
        return upload_video_to_service_canister(
            admin_ic_agent,
            PostServicePostDetailsFromFrontend {
                hashtags: post_details.hashtags,
                description: post_details.description,
                video_uid: post_details.video_uid,
                creator_principal: user_ic_agent.get_principal().map_err(|e| {
                    console_error!("Error getting creator principal. Error {}", e);
                    e
                })?,
                id: Uuid::new_v4().to_string(),
            },
        )
        .await;
    }
}

pub async fn upload_video(
    events: &EventService,
    video_uid: String,
    user_ic_agent: &Agent,
    admin_ic_agent: &Agent,
    post_details: PostDetailsFromFrontend,
    country: Option<String>,
) -> Result<(), Box<dyn Error>> {
    match upload_video_to_canister(user_ic_agent, admin_ic_agent, post_details.clone()).await {
        Ok(post_id) => {
            console_log!("video upload to canister successful");

            let _ = events
                .send_video_upload_successful_event(
                    video_uid,
                    post_details.hashtags.len(),
                    post_details.is_nsfw,
                    post_details.creator_consent_for_inclusion_in_hot_or_not,
                    post_id,
                    user_ic_agent.get_principal()?,
                    Principal::anonymous(),
                    String::new(),
                    country,
                )
                .await
                .inspect_err(|e| {
                    console_error!(
                        "Error sending video successful event. Error {}",
                        e.to_string()
                    )
                });

            Ok(())
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
                    user_ic_agent.get_principal()?,
                    String::new(),
                    Principal::anonymous(),
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

async fn upload_video_to_canister(
    user_ic_agent: &Agent,
    admin_ic_agent: &Agent,
    post_details: PostDetailsFromFrontend,
) -> Result<String, Box<dyn Error>> {
    let post_id =
        upload_video_to_canister_impl(user_ic_agent, admin_ic_agent, post_details).await?;

    Ok(post_id)
}

pub async fn mark_video_as_downloadable(
    cloudflare_stream: &CloudflareStream,
    video_uid: &str,
) -> Result<(), Box<dyn Error>> {
    cloudflare_stream
        .mark_video_as_downloadable(video_uid)
        .await?;
    Ok(())
}

async fn upload_video_to_individual_canister(
    individual_user_canister: &IndividualUserCanisterService<'_>,
    post_details: PostDetailsFromFrontend,
) -> Result<u64, Box<dyn Error>> {
    let result = individual_user_canister.add_post_v_2(post_details).await?;
    match result {
        AddPostResult::Ok(post_id) => Ok(post_id),
        AddPostResult::Err(err) => Err(err.into()),
    }
}

async fn upload_video_to_service_canister(
    admin_ic_agent: &Agent,
    post_details: PostServicePostDetailsFromFrontend,
) -> Result<String, Box<dyn Error>> {
    let user_info_service = UserInfoService(USER_INFO_SERVICE_ID, admin_ic_agent);

    let user_details = user_info_service
        .get_user_profile_details(post_details.creator_principal)
        .await?;

    let post_id = post_details.id;

    if let Result1::Ok(_user_details) = user_details {
        let post_service = UserPostService(USER_POST_SERVICE_ID, admin_ic_agent);
        let result = post_service
            .add_post(PostServicePostDetailsFromFrontend {
                id: post_id.clone(),
                hashtags: post_details.hashtags,
                description: post_details.description,
                video_uid: post_details.video_uid,
                creator_principal: post_details.creator_principal,
            })
            .await?;

        match result {
            Result_::Ok => Ok(post_id),
            Result_::Err(e) => Err(format!("{e:?}").into()),
        }
    } else {
        Err("User details not found".into())
    }
}
