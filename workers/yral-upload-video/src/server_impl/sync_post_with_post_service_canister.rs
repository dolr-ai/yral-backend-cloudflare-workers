use candid::Principal;
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use std::error::Error;
use yral_canisters_client::{
    ic::USER_POST_SERVICE_ID,
    individual_user_template::{IndividualUserTemplate, Post, PostStatus, Result4},
    user_post_service::{
        Post as PostForPostServiceSync, PostStatus as PostServicePostStatus, PostViewStatistics,
        SystemTime, UserPostService,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SyncPostToPostServiceRequest {
    user_principal: Principal,
    canister_id: Principal,
    post_id: u64,
}

pub async fn fetch_post_from_individual_canister(
    agent: &Agent,
    canister_id: Principal,
    post_id: u64,
) -> Result<Post, Box<dyn Error>> {
    let individual_user_service = IndividualUserTemplate(canister_id, agent);

    let entire_post_result = individual_user_service
        .get_entire_individual_post_detail_by_id(post_id)
        .await?;

    match entire_post_result {
        Result4::Ok(post) => Ok(post),
        Result4::Err => Err("Unauthorized".into()),
    }
}

pub async fn sync_post_with_post_service_canister_impl(
    agent: &Agent,
    sync_post_req: SyncPostToPostServiceRequest,
) -> Result<(), Box<dyn Error>> {
    let post_from_individual_canister = fetch_post_from_individual_canister(
        agent,
        sync_post_req.canister_id,
        sync_post_req.post_id,
    )
    .await?;

    let post_service_canister = UserPostService(USER_POST_SERVICE_ID, agent);

    let uuid = uuid::Uuid::new_v4().to_string();

    let sync_post_created_at = SystemTime {
        nanos_since_epoch: post_from_individual_canister.created_at.nanos_since_epoch,
        secs_since_epoch: post_from_individual_canister.created_at.secs_since_epoch,
    };

    // Convert PostStatus from individual_user_template to user_post_service
    let sync_post_status = match post_from_individual_canister.status {
        PostStatus::BannedDueToUserReporting => PostServicePostStatus::BannedDueToUserReporting,
        PostStatus::CheckingExplicitness => PostServicePostStatus::CheckingExplicitness,
        PostStatus::Deleted => PostServicePostStatus::Deleted,
        PostStatus::BannedForExplicitness => PostServicePostStatus::BannedForExplicitness,
        PostStatus::Uploaded => PostServicePostStatus::Uploaded,
        PostStatus::ReadyToView => PostServicePostStatus::ReadyToView,
        PostStatus::Transcoding => PostServicePostStatus::Transcoding,
    };

    let sync_post_view_stats = PostViewStatistics {
        total_view_count: post_from_individual_canister.view_stats.total_view_count,
        average_watch_percentage: post_from_individual_canister
            .view_stats
            .average_watch_percentage,
        threshold_view_count: post_from_individual_canister
            .view_stats
            .threshold_view_count,
    };

    let sync_post_from_individual_canister = PostForPostServiceSync {
        id: uuid,
        creator_principal: sync_post_req.user_principal,
        video_uid: post_from_individual_canister.video_uid,
        description: post_from_individual_canister.description,
        hashtags: post_from_individual_canister.hashtags,
        created_at: sync_post_created_at,
        likes: post_from_individual_canister.likes,
        status: sync_post_status,
        share_count: post_from_individual_canister.share_count,
        view_stats: sync_post_view_stats,
    };

    let _ = post_service_canister
        .sync_post_from_individual_canister(sync_post_from_individual_canister)
        .await?;

    Ok(())
}
