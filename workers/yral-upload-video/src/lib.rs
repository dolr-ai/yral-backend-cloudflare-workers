use axum::body::Body;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    debug_handler,
    routing::{get, post},
    Json, Router,
};
use ic_agent::identity::DelegatedIdentity;
use ic_agent::Agent;
use serde::{Deserialize, Serialize};
use server_impl::upload_video_to_canister::{
    upload_video_to_canister, UploadVideoToCanisterResult,
};
use std::collections::HashMap;
use std::fmt::Display;
use std::result::Result;
use std::{error::Error, sync::Arc};
use tower_http::cors::CorsLayer;
use tower_service::Service;
use utils::cloudflare_stream::CloudflareStream;
use utils::events::{EventService, Warehouse};
use utils::individual_user_canister::PostDetailsFromFrontend;
use utils::notification::{NotificationClient, NotificationType};
use utils::types::{
    DelegatedIdentityWire, DirectUploadResult, Video, DELEGATED_IDENTITY_KEY, POST_DETAILS_KEY,
};
use utils::user_ic_agent::create_ic_agent_from_meta;
use worker::Result as WorkerResult;
use worker::*;

use axum::extract::State;

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
                message: Some(format!("{}", err.to_string())),
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
}

impl AppState {
    fn new(
        clouflare_account_id: String,
        cloudflare_api_token: String,
        webhook_secret_key: String,
        off_chain_auth_token: String,
        upload_video_queue: Queue,
    ) -> Result<Self, Box<dyn Error>> {
        let cloudflare_stream = CloudflareStream::new(clouflare_account_id, cloudflare_api_token)?;
        Ok(Self {
            cloudflare_stream,
            events: Warehouse::with_auth_token(off_chain_auth_token.clone()),
            webhook_secret_key,
            event_rest_service: EventService::with_auth_token(off_chain_auth_token),
            upload_video_queue,
        })
    }
}

fn router(env: Env, ctx: Context) -> Router {
    let upload_queue: Queue = env.queue("UPLOAD_VIDEO").expect("Queue binding invalid");

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
        env.secret("OFF_CHAIN_GRPC_AUTH_TOKEN").unwrap().to_string(),
        upload_queue,
    )
    .unwrap();

    Router::new()
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
    message_batch: MessageBatch<String>,
    env: Env,
    _: Context,
) -> Result<(), Box<dyn Error>> {
    let cloudflare_stream_client = CloudflareStream::new(
        env.secret("CLOUDFLARE_STREAM_ACCOUNT_ID")?.to_string(),
        env.secret("CLOUDFLARE_STREAM_API_TOKEN")?.to_string(),
    )?;

    let events_rest_service =
        EventService::with_auth_token(env.secret("OFF_CHAIN_GRPC_AUTH_TOKEN")?.to_string());

    let notif_client = NotificationClient::new(
        env.secret("YRAL_METADATA_USER_NOTIFICATION_API_KEY")?
            .to_string(),
    );

    for message in message_batch.messages()? {
        process_message(
            message,
            &cloudflare_stream_client,
            &events_rest_service,
            &notif_client,
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
    message: Message<String>,
    cloudflare_stream_client: &CloudflareStream,
    events_rest_service: &EventService,
    notif_client: &NotificationClient,
) {
    let video_uid = message.body();
    let video_details_result = cloudflare_stream_client.get_video_details(video_uid).await;

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

    let Ok(ic_agent) = create_ic_agent_from_meta(meta) else {
        console_error!("error creating ic agent");
        message.retry();
        return;
    };

    match is_video_ready {
        Ok((true, _)) => {
            let result = extract_fields_from_video_meta_and_upload_video(
                &cloudflare_stream_client,
                video_uid.to_string(),
                meta,
                events_rest_service,
                &ic_agent,
            )
            .await;

            match result {
                Ok(post_meta) => {
                    notif_client
                        .send_notification(
                            NotificationType::VideoUploadSuccess(post_meta),
                            ic_agent.get_principal().ok(),
                        )
                        .await;

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

            notif_client
                .send_notification(
                    NotificationType::VideoUploadError,
                    ic_agent.get_principal().ok(),
                )
                .await;

            message.ack();
        }
        Err(e) => {
            console_error!("Error extracting video status. Error {}", e.to_string());
            message.retry();
        }
    };
}

pub async fn extract_fields_from_video_meta_and_upload_video(
    cloudflare_stream: &CloudflareStream,
    video_uid: String,
    meta: &HashMap<String, String>,
    events: &EventService,
    agent: &Agent,
) -> Result<UploadVideoToCanisterResult, Box<dyn Error>> {
    let post_details_from_frontend_string = meta
        .get(POST_DETAILS_KEY)
        .ok_or("post details not found in meta")?;

    let country = meta.get("country").cloned();

    let post_details_from_frontend: PostDetailsFromFrontend =
        serde_json::from_str(post_details_from_frontend_string)?;

    upload_video_to_canister(
        cloudflare_stream,
        events,
        video_uid,
        agent,
        post_details_from_frontend,
        country,
    )
    .await
}

pub async fn root() -> &'static str {
    "Hello Axum!"
}

#[derive(Serialize, Deserialize)]
struct UpdateMetadataRequest {
    video_uid: String,
    delegated_identity_wire: DelegatedIdentityWire,
    meta: HashMap<String, String>,
    post_details: PostDetailsFromFrontend,
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

    // upload video uid
    let queue_send_result = app_state.upload_video_queue.send(video_uid).await;

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
pub async fn notify_video_upload(payload: String) -> APIResponse<()> {
    console_log!("Notify Recieved: {:?}", &payload);

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
        serde_json::to_string(&req_data.post_details)?,
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

async fn get_upload_url_impl_v2(
    cloudflare_stream: &CloudflareStream,
) -> Result<DirectUploadResult, Box<dyn Error>> {
    let result = cloudflare_stream.get_upload_url_v2().await?;
    Ok(result)
}
