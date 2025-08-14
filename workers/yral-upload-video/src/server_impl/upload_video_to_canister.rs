use std::error::Error;

use candid::Principal;
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use worker::{console_error, console_log};
use yral_canisters_client::{
    individual_user_template::{
        IndividualUserTemplate as IndividualUserCanisterService, PostDetailsFromFrontend,
        Result1 as AddPostResult,
    },
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

static USER_INFO_SERVICE_CANISTER_ID: &str = "ivkka-7qaaa-aaaas-qbg3q-cai";
static USER_POST_SERVICE_CANISTER_ID: &str = "gxhc3-pqaaa-aaaas-qbh3q-cai";

pub async fn upload_video_to_canister_impl(
    user_ic_agent: &Agent,
    admin_ic_agent: &Agent,
    post_details: PostDetailsFromFrontend,
) -> Result<(), Box<dyn Error>> {
    let yral_metadata_client = yral_metadata_client::MetadataClient::default();

    let user_details = yral_metadata_client
        .get_user_metadata_v2(user_ic_agent.get_principal()?.to_string())
        .await?;

    if let Some(user_details) = user_details {
        let individual_user_service =
            IndividualUserCanisterService(user_details.user_canister_id, user_ic_agent);

        upload_video_to_individual_canister(&individual_user_service, post_details).await?;
    } else {
        return upload_video_to_service_canister(
            admin_ic_agent,
            PostServicePostDetailsFromFrontend {
                hashtags: post_details.hashtags,
                description: post_details.description,
                video_uid: post_details.video_uid,
                creator_principal: user_ic_agent.get_principal().unwrap(),
                id: Uuid::now_v7().to_string(),
            },
        )
        .await;
    }
    Ok(())
}

pub async fn upload_video_to_canister(
    cloudflare_stream: &CloudflareStream,
    events: &EventService,
    video_uid: String,
    user_ic_agent: &Agent,
    admin_ic_agent: &Agent,
    post_details: PostDetailsFromFrontend,
    country: Option<String>,
) -> Result<(), Box<dyn Error>> {
    match upload_video_to_canister_and_mark_video_for_download(
        cloudflare_stream,
        &video_uid,
        user_ic_agent,
        admin_ic_agent,
        post_details.clone(),
    )
    .await
    {
        Ok(_) => {
            console_log!("video upload to canister successful");

            let _ = events
                .send_video_upload_successful_event(
                    video_uid,
                    post_details.hashtags.len(),
                    post_details.is_nsfw,
                    post_details.creator_consent_for_inclusion_in_hot_or_not,
                    0,
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

async fn upload_video_to_canister_and_mark_video_for_download(
    cloudflare_stream: &CloudflareStream,
    video_uid: &str,
    user_ic_agent: &Agent,
    admin_ic_agent: &Agent,
    post_details: PostDetailsFromFrontend,
) -> Result<(), Box<dyn Error>> {
    upload_video_to_canister_impl(user_ic_agent, admin_ic_agent, post_details).await?;

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
) -> Result<(), Box<dyn Error>> {
    let user_info_service = UserInfoService(
        Principal::from_text(USER_INFO_SERVICE_CANISTER_ID).unwrap(),
        admin_ic_agent,
    );

    let user_details = user_info_service
        .get_user_profile_details(post_details.creator_principal)
        .await?;

    if let Result1::Ok(_user_details) = user_details {
        let post_service = UserPostService(
            Principal::from_text(USER_POST_SERVICE_CANISTER_ID).unwrap(),
            admin_ic_agent,
        );
        let result = post_service
            .add_post(PostServicePostDetailsFromFrontend {
                id: Uuid::now_v7().to_string(),
                hashtags: post_details.hashtags,
                description: post_details.description,
                video_uid: post_details.video_uid,
                creator_principal: post_details.creator_principal,
            })
            .await?;

        match result {
            Result_::Ok => Ok(()),
            Result_::Err(e) => Err(format!("{e:?}").into()),
        }
    } else {
        Err("User details not found".into())
    }
}
