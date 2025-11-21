use axum::body::Body;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::IntoResponse;
use axum::{
    debug_handler,
    routing::{get, post},
    Json, Router,
};
use candid::Principal;
use ic_agent::identity::{DelegatedIdentity, Secp256k1Identity};
use ic_agent::Agent;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::result::Result;
use std::{error::Error, sync::Arc};
use tower_http::cors::CorsLayer;
use tower_service::Service;
use utils::cloudflare_stream::CloudflareStream;
use utils::events::{EventService, Warehouse};
use utils::types::{
    DelegatedIdentityWire, DirectUploadResult, Video, DELEGATED_IDENTITY_KEY, POST_DETAILS_KEY,
};
use utils::user_ic_agent::create_ic_agent_from_meta;
use worker::Result as WorkerResult;
use worker::*;
use yral_canisters_client::individual_user_template::PostDetailsFromFrontend;

use axum::extract::State;

use crate::server_impl::notify_video_upload_impl::notify_video_upload_impl;
use crate::server_impl::sync_post_with_post_service_canister::SyncPostToPostServiceRequest;
use crate::server_impl::upload_video_to_canister::{
    mark_post_as_published_and_emit_events, upload_video,
};
use crate::server_impl::{
    sync_post_with_post_service_canister::sync_post_with_post_service_canister_impl,
    upload_video_to_canister::mark_video_as_downloadable,
};
use crate::utils::notification_client::NotificationClient;
use crate::utils::service_canister_post_mapping_redis_rest_client::RedisRestClient;
use crate::utils::types::{MarkPostAsPublishedRequest, RequestPostDetails};

pub mod server_impl;
pub mod utils;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct APIResponse<T>
where
    T: Clone + Serialize,
{
    pub message: Option<String>,
    pub success: bool,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UploadVideoQueueMessage {
    UploadVideo(String),
    MarkVideoAsDownloadable(String),
    PushPostToPostServiceCanister(SyncPostToPostServiceRequest),
}

impl<T> IntoResponse for APIResponse<T>
where
    T: Clone + Serialize,
{
    fn into_response(self) -> axum::response::Response<Body> {
        let mut response_body = Json(APIResponse {
            message: self.message.clone(),
            success: self.success,
            data: self.data,
        })
        .into_response();

        if !self.success {
            *response_body.status_mut() = StatusCode::BAD_REQUEST;
        }

        response_body
    }
}

#[derive(Serialize, Deserialize)]
pub struct VideoKvStoreValue {
    pub user_delegated_identity_wire: Option<DelegatedIdentityWire>,
    pub meta: Option<HashMap<String, String>>,
    pub direct_upload_result: DirectUploadResult,
}

impl<T, E> From<Result<T, E>> for APIResponse<T>
where
    E: Display,
    T: Clone + Serialize,
{
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(data) => Self {
                message: None,
                success: true,
                data: Some(data),
            },
            Err(err) => Self {
                message: Some(format!("{err}")),
                success: false,
                data: None,
            },
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub cloudflare_stream: CloudflareStream,
    pub events: Warehouse,
    pub webhook_secret_key: String,
    pub event_rest_service: EventService,
    pub upload_video_queue: Queue,
    pub admin_ic_agent: Agent,
    pub notification_client: NotificationClient,
}

impl AppState {
    fn new(
        clouflare_account_id: String,
        cloudflare_api_token: String,
        webhook_secret_key: String,
        off_chain_auth_token: String,
        upload_video_queue: Queue,
        canisters_admin_key: String,
        notification_api_key: String,
    ) -> Result<Self, Box<dyn Error>> {
        let cloudflare_stream = CloudflareStream::new(clouflare_account_id, cloudflare_api_token)?;
        let notification_client = NotificationClient::new(notification_api_key);
        Ok(Self {
            cloudflare_stream,
            events: Warehouse::with_auth_token(off_chain_auth_token.clone()),
            webhook_secret_key,
            event_rest_service: EventService::with_auth_token(off_chain_auth_token),
            upload_video_queue,
            admin_ic_agent: init_canisters_admin_ic_agent(canisters_admin_key)?,
            notification_client,
        })
    }
}

fn init_canisters_admin_ic_agent(identity_str: String) -> Result<Agent, Box<dyn Error>> {
    let identity = Secp256k1Identity::from_pem(identity_str.as_bytes())?;

    let agent = Agent::builder()
        .with_identity(identity)
        .with_url("https://ic0.app/")
        .build()?;

    Ok(agent)
}

fn router(env: Env, _ctx: Context) -> Router {
    let upload_queue: Queue = env.queue("UPLOAD_VIDEO").expect("Queue binding invalid");
    let off_chain_auth_token = env.secret("OFF_CHAIN_GRPC_AUTH_TOKEN").unwrap().to_string();
    let notification_api_key = env
        .secret("YRAL_METADATA_USER_NOTIFICATION_API_KEY")
        .unwrap()
        .to_string();

    let off_chain_auth_token_clone = off_chain_auth_token.clone();

    let app_state = AppState::new(
        env.secret("CLOUDFLARE_STREAM_ACCOUNT_ID")
            .unwrap()
            .to_string(),
        env.secret("CLOUDFLARE_STREAM_API_TOKEN")
            .unwrap()
            .to_string(),
        env.secret("CLOUDFLARE_STREAM_WEBHOOK_SECRET")
            .unwrap()
            .to_string(),
        off_chain_auth_token.clone(),
        upload_queue,
        env.secret("CANISTERS_ADMIN_KEY").unwrap().to_string(),
        notification_api_key,
    )
    .unwrap();

    Router::new()
        .route(
            "/sync_post_to_post_canister",
            post(sync_post_with_post_service_canister),
        )
        .route(
            "/create_video_url_for_ai_draft",
            post(get_upload_url_for_ai_draft_video),
        )
        .route_layer(middleware::from_fn(
            move |req: axum::http::Request<Body>, next: Next| {
                let auth_token = off_chain_auth_token_clone.clone();
                async move {
                    let headers = req.headers();
                    let auth_header = headers.get(AUTHORIZATION);
                    if let Some(header_value) = auth_header {
                        let auth_str = header_value.to_str().unwrap_or("");
                        if auth_str != format!("Bearer {}", auth_token) {
                            return Err(StatusCode::UNAUTHORIZED);
                        }
                    } else {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                    Ok::<_, StatusCode>(next.run(req).await)
                }
            },
        ))
        .route("/mark_post_as_published", post(mark_post_as_published))
        .route("/", get(root))
        .route("/get_upload_url", get(get_upload_url))
        .route("/get_upload_url_v2", get(get_upload_url_v2))
        .route("/update_metadata", post(update_metadata))
        .route("/notify", post(notify_video_upload))
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(app_state))
}

#[event(fetch)]
async fn fetch(
    req: HttpRequest,
    env: Env,
    ctx: Context,
) -> WorkerResult<axum::http::Response<axum::body::Body>> {
    console_error_panic_hook::set_once();
    Ok(router(env, ctx).call(req).await?)
}

#[event(queue)]
async fn queue(
    message_batch: MessageBatch<UploadVideoQueueMessage>,
    env: Env,
    _: Context,
) -> Result<(), Box<dyn Error>> {
    let cloudflare_stream_client = CloudflareStream::new(
        env.secret("CLOUDFLARE_STREAM_ACCOUNT_ID")?.to_string(),
        env.secret("CLOUDFLARE_STREAM_API_TOKEN")?.to_string(),
    )?;

    let upload_queue: Queue = env.queue("UPLOAD_VIDEO").expect("Queue binding invalid");

    let admin_ic_agent =
        init_canisters_admin_ic_agent(env.secret("CANISTERS_ADMIN_KEY")?.to_string())?;

    let events_rest_service =
        EventService::with_auth_token(env.secret("OFF_CHAIN_GRPC_AUTH_TOKEN")?.to_string());

    let service_canister_post_mapping_redis_rest_endpoint = env
        .secret("SERVICE_CANISTER_POST_MAPPING_REDIS_REST_ENDPOINT")?
        .to_string();
    let service_canister_post_mapping_redis_rest_token = env
        .secret("SERVICE_CANISTER_POST_MAPPING_REDIS_REST_TOKEN")?
        .to_string();

    let service_canister_post_mapping_client = RedisRestClient::new(
        service_canister_post_mapping_redis_rest_endpoint,
        service_canister_post_mapping_redis_rest_token,
    )?;

    for message in message_batch.messages()? {
        process_message(
            message,
            &upload_queue,
            &cloudflare_stream_client,
            &events_rest_service,
            &admin_ic_agent,
            &service_canister_post_mapping_client,
        )
        .await;
    }

    Ok(())
}

fn is_video_ready(video_details: &Video) -> Result<(bool, String), Box<dyn Error>> {
    let video_status = video_details
        .status
        .as_ref()
        .ok_or("video status not found")?;

    let video_state = video_status.state.as_ref().ok_or("video state not found")?;

    if video_state.eq("error") {
        Ok((
            false,
            video_status
                .err_reason_text
                .as_ref()
                .cloned()
                .unwrap_or_default(),
        ))
    } else if video_state.eq("ready") {
        Ok((true, String::new()))
    } else {
        Err("video still processing".into())
    }
}

pub async fn process_message(
    message: Message<UploadVideoQueueMessage>,
    upload_queue: &Queue,
    cloudflare_stream_client: &CloudflareStream,
    events_rest_service: &EventService,
    admin_ic_agent: &Agent,
    service_canister_post_mapping_client: &RedisRestClient,
) {
    let message_body = message.body();

    match message_body {
        UploadVideoQueueMessage::UploadVideo(video_uid) => {
            process_message_for_video_upload(
                &message,
                upload_queue,
                cloudflare_stream_client,
                events_rest_service,
                admin_ic_agent,
                video_uid.clone(),
            )
            .await;
        }
        UploadVideoQueueMessage::MarkVideoAsDownloadable(video_uid) => {
            process_message_for_marking_video_downloadable(
                &message,
                cloudflare_stream_client,
                video_uid.clone(),
            )
            .await;
        }
        UploadVideoQueueMessage::PushPostToPostServiceCanister(request_payload) => {
            process_message_for_sync_video_to_post_service_canister(
                &message,
                admin_ic_agent,
                request_payload.clone(),
                service_canister_post_mapping_client,
            )
            .await;
        }
    }
}

async fn process_message_for_sync_video_to_post_service_canister(
    message: &Message<UploadVideoQueueMessage>,
    admin_ic_agent: &Agent,
    sync_video_request: SyncPostToPostServiceRequest,
    service_canister_post_mapping_client: &RedisRestClient,
) {
    match sync_post_with_post_service_canister_impl(
        admin_ic_agent,
        sync_video_request,
        service_canister_post_mapping_client,
    )
    .await
    {
        Ok(_) => message.ack(),
        Err(e) => {
            console_error!("Error syncing post to post service canister: {}", e);
            message.retry();
        }
    }
}

pub async fn process_message_for_video_upload(
    message: &Message<UploadVideoQueueMessage>,
    upload_queue: &Queue,
    cloudflare_stream_client: &CloudflareStream,
    events_rest_service: &EventService,
    admin_ic_agent: &Agent,
    video_uid: String,
) {
    let video_details_result = cloudflare_stream_client.get_video_details(&video_uid).await;

    if let Err(e) = video_details_result.as_ref() {
        console_error!("Error {}", e.to_string());
        message.retry();
        return;
    }

    let video_details = video_details_result.unwrap();

    let is_video_ready = is_video_ready(&video_details);

    let Ok(meta) = video_details.meta.as_ref().ok_or("meta not found") else {
        console_error!("meta not found");
        message.retry();
        return;
    };

    match is_video_ready {
        Ok((true, _)) => {
            let result = extract_fields_from_video_meta_and_upload_video(
                video_uid.to_string(),
                meta,
                events_rest_service,
                admin_ic_agent,
            )
            .await;

            match result {
                Ok(_post_meta) => {
                    let mark_video_download_message =
                        UploadVideoQueueMessage::MarkVideoAsDownloadable(video_uid.to_string());
                    if let Err(e) = upload_queue.send(mark_video_download_message).await {
                        console_log!("Error sending mark video download message: {}", e);
                    }
                    message.ack();
                }
                Err(e) => {
                    console_error!(
                        "Error uploading video {} to canister {}",
                        video_uid,
                        e.to_string()
                    );

                    message.retry()
                }
            }
        }
        Ok((false, err)) => {
            console_error!(
                "Error processing video {} on cloudflare. Error {}",
                video_uid,
                err
            );

            message.ack();
        }
        Err(e) => {
            console_error!("Error extracting video status. Error {}", e.to_string());
            message.retry();
        }
    };
}

pub async fn process_message_for_marking_video_downloadable(
    message: &Message<UploadVideoQueueMessage>,
    cloudflare_stream_client: &CloudflareStream,
    video_uid: String,
) {
    if let Err(e) = mark_video_as_downloadable(cloudflare_stream_client, &video_uid).await {
        console_error!("Error marking video {} as downloadable: {}", video_uid, e);
        message.retry();
    }
    message.ack();
}

pub async fn extract_fields_from_video_meta_and_upload_video(
    video_uid: String,
    meta: &HashMap<String, String>,
    events: &EventService,
    admin_ic_agent: &Agent,
) -> Result<(), Box<dyn Error>> {
    let post_details_from_frontend_string = meta
        .get(POST_DETAILS_KEY)
        .ok_or("post details not found in meta")?;

    let country = meta.get("country").cloned();

    let post_details_from_frontend: RequestPostDetails =
        serde_json::from_str(post_details_from_frontend_string)?;

    let user_agent = create_ic_agent_from_meta(meta)?;

    upload_video(
        events,
        video_uid,
        &user_agent,
        admin_ic_agent,
        post_details_from_frontend.into(),
        country,
    )
    .await
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}

#[derive(Debug, Clone, Deserialize)]
struct UpdateMetadataRequest {
    video_uid: String,
    delegated_identity_wire: DelegatedIdentityWire,
    meta: HashMap<String, String>,
    post_details: PostDetailsFromFrontend,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct AIVideoUploadUrlRequest {
    pub user_id: Principal,
}

#[debug_handler]
#[worker::send]
pub async fn sync_post_with_post_service_canister(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<SyncPostToPostServiceRequest>,
) -> APIResponse<()> {
    let sync_post_message = UploadVideoQueueMessage::PushPostToPostServiceCanister(payload);

    let message_result = app_state.upload_video_queue.send(sync_post_message).await;

    message_result.into()
}

#[debug_handler]
#[worker::send]
pub async fn mark_post_as_published(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<MarkPostAsPublishedRequest>,
) -> APIResponse<()> {
    let user_principal =
        Principal::self_authenticating(payload.delegated_identity_wire.from_key.clone());

    let post_id = payload.post_id.clone();

    let result = mark_post_as_published_and_emit_events(
        &app_state.admin_ic_agent,
        &app_state.event_rest_service,
        payload,
    )
    .await;

    if let Ok(()) = &result {
        app_state
            .notification_client
            .send_notification(
                utils::notification_client::NotificationType::VideoPublished {
                    user_principal,
                    post_id,
                },
                Some(user_principal),
            )
            .await;
    }

    result.into()
}

#[debug_handler]
#[worker::send]
pub async fn update_metadata(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<UpdateMetadataRequest>,
) -> APIResponse<()> {
    let video_uid = payload.video_uid.clone();
    let result = update_metadata_impl(&app_state.cloudflare_stream, payload).await;

    let api_response: APIResponse<()> = result.into();

    if !api_response.success {
        console_error!(
            "error updating metadata {}",
            &api_response.message.as_ref().unwrap_or(&String::from(""))
        )
    }

    let upload_video_message = UploadVideoQueueMessage::UploadVideo(video_uid.clone());

    // upload video uid
    let queue_send_result = app_state
        .upload_video_queue
        .send(upload_video_message)
        .await;

    if let Err(e) = queue_send_result {
        console_error!(
            "Error sending message to upload queue. Error {}",
            e.to_string()
        );
    }

    api_response
}

#[debug_handler]
#[worker::send]
pub async fn notify_video_upload(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    payload: String,
) -> APIResponse<()> {
    console_log!("Notify Recieved");

    let webhook_secret_key = app_state.webhook_secret_key.clone();

    if let Err(e) = notify_video_upload_impl(
        &app_state.admin_ic_agent,
        &app_state.notification_client,
        payload,
        headers,
        webhook_secret_key,
    )
    .await
    {
        console_error!("Error in notify video upload. Error {}", e.to_string());
    } else {
        console_log!("Notify Processed Successfully");
    }

    Ok::<(), Box<dyn Error>>(()).into()
}

async fn update_metadata_impl(
    cloudflare_stream: &CloudflareStream,
    mut req_data: UpdateMetadataRequest,
) -> Result<(), Box<dyn Error>> {
    let _delegated_identity =
        DelegatedIdentity::try_from(req_data.delegated_identity_wire.clone())?;

    req_data.meta.insert(
        DELEGATED_IDENTITY_KEY.to_string(),
        serde_json::to_string(&req_data.delegated_identity_wire)?,
    );

    req_data.meta.insert(
        POST_DETAILS_KEY.to_string(),
        serde_json::to_string(&Into::<RequestPostDetails>::into(req_data.post_details))?,
    );

    cloudflare_stream
        .add_meta_to_video(&req_data.video_uid, req_data.meta)
        .await?;

    Ok(())
}

#[debug_handler]
#[worker::send]
pub async fn get_upload_url(
    State(app_state): State<Arc<AppState>>,
) -> APIResponse<DirectUploadResult> {
    get_upload_url_impl(&app_state.cloudflare_stream)
        .await
        .into()
}

async fn get_upload_url_impl(
    cloudflare_stream: &CloudflareStream,
) -> Result<DirectUploadResult, Box<dyn Error>> {
    let result = cloudflare_stream.get_upload_url().await?;
    Ok(result)
}

#[debug_handler]
#[worker::send]
pub async fn get_upload_url_v2(
    State(app_state): State<Arc<AppState>>,
) -> APIResponse<DirectUploadResult> {
    get_upload_url_impl_v2(&app_state.cloudflare_stream)
        .await
        .into()
}

#[debug_handler]
#[worker::send]
pub async fn get_upload_url_for_ai_draft_video(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<AIVideoUploadUrlRequest>,
) -> APIResponse<DirectUploadResult> {
    get_upload_url_for_ai_draft_video_impl(&app_state.cloudflare_stream, payload.user_id.to_text())
        .await
        .into()
}

async fn get_upload_url_for_ai_draft_video_impl(
    cloudflare_stream: &CloudflareStream,
    user_principal: String,
) -> Result<DirectUploadResult, Box<dyn Error>> {
    let result = cloudflare_stream
        .get_upload_url_for_ai_draft_video(user_principal)
        .await?;
    Ok(result)
}

async fn get_upload_url_impl_v2(
    cloudflare_stream: &CloudflareStream,
) -> Result<DirectUploadResult, Box<dyn Error>> {
    let result = cloudflare_stream.get_upload_url_v2().await?;
    Ok(result)
}
