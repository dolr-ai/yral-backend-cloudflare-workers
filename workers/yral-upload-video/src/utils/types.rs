use std::{collections::HashMap, error::Error};

use ic_agent::identity::{DelegatedIdentity, Secp256k1Identity, SignedDelegation};
use k256::elliptic_curve::JwkEcKey;
use serde::{Deserialize, Serialize};
use yral_canisters_client::individual_user_template::PostDetailsFromFrontend;

pub const DELEGATED_IDENTITY_KEY: &str = "delegated-identity";
pub const POST_DETAILS_KEY: &str = "post-details";
pub const CF_WATERMARK_UID: &str = "b5588fa1516ca33a08ebfef06c8edb33";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestPostDetails {
    pub video_uid: String,
    pub description: String,
    pub is_nsfw: bool,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
    pub hashtags: Vec<String>,
}

impl From<PostDetailsFromFrontend> for RequestPostDetails {
    fn from(value: PostDetailsFromFrontend) -> Self {
        Self {
            video_uid: value.video_uid,
            description: value.description,
            is_nsfw: value.is_nsfw,
            hashtags: value.hashtags,
            creator_consent_for_inclusion_in_hot_or_not: value
                .creator_consent_for_inclusion_in_hot_or_not,
        }
    }
}

impl From<RequestPostDetails> for PostDetailsFromFrontend {
    fn from(value: RequestPostDetails) -> Self {
        Self {
            video_uid: value.video_uid,
            description: value.description,
            is_nsfw: value.is_nsfw,
            hashtags: value.hashtags,
            creator_consent_for_inclusion_in_hot_or_not: value
                .creator_consent_for_inclusion_in_hot_or_not,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotifyStatusType {
    pub state: Option<String>,
    pub step: Option<String>,
    #[serde(rename = "pctComplete")]
    pub pct_complete: Option<String>,
    #[serde(rename = "errorReasonCode")]
    pub err_reason_code: Option<String>,
    #[serde(rename = "errorReasonText")]
    pub err_reason_text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NotifyRequestPayload {
    pub uid: String,
    #[serde(rename = "readyToStream")]
    pub ready_to_stream: bool,
    pub status: NotifyStatusType,
    pub meta: HashMap<String, String>,
    pub created: Option<String>,
    pub modified: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Video {
    #[serde(rename = "allowedOrigins", skip_serializing_if = "Option::is_none")]
    pub allowed_origins: Option<Vec<String>>, // Array<AllowedOrigins>

    #[serde(rename = "created", skip_serializing_if = "Option::is_none")]
    pub created: Option<String>, // format: date-time

    #[serde(rename = "creator", skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>, // maxLength: 64

    #[serde(rename = "duration", skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>, // number, -1 means unknown

    #[serde(rename = "input", skip_serializing_if = "Option::is_none")]
    pub input: Option<Input>, // { height, width }

    #[serde(rename = "liveInput", skip_serializing_if = "Option::is_none")]
    pub live_input: Option<String>, // maxLength: 32

    #[serde(rename = "maxDurationSeconds", skip_serializing_if = "Option::is_none")]
    pub max_duration_seconds: Option<i32>, // maximum: 21600, minimum: 1, -1 means unknown

    #[serde(rename = "meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, String>>, // unknown, key-value store

    #[serde(rename = "modified", skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>, // format: date-time

    #[serde(rename = "playback", skip_serializing_if = "Option::is_none")]
    pub playback: Option<Playback>, // { dash, hls }

    #[serde(rename = "preview", skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>, // format: uri

    #[serde(rename = "readyToStream", skip_serializing_if = "Option::is_none")]
    pub ready_to_stream: Option<bool>, // boolean

    #[serde(rename = "readyToStreamAt", skip_serializing_if = "Option::is_none")]
    pub ready_to_stream_at: Option<String>, // format: date-time

    #[serde(rename = "requireSignedURLs", skip_serializing_if = "Option::is_none")]
    pub require_signed_urls: Option<bool>, // boolean

    #[serde(rename = "scheduledDeletion", skip_serializing_if = "Option::is_none")]
    pub scheduled_deletion: Option<String>, // format: date-time

    #[serde(rename = "size", skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>, // number (size in bytes)

    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<NotifyStatusType>, // { errorReasonCode, errorReasonText, pctComplete, 1 more... }

    #[serde(rename = "thumbnail", skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>, // format: uri

    #[serde(
        rename = "thumbnailTimestampPct",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail_timestamp_pct: Option<f32>, // maximum: 1, minimum: 0

    #[serde(rename = "uid", skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>, // maxLength: 32

    #[serde(rename = "uploaded", skip_serializing_if = "Option::is_none")]
    pub uploaded: Option<String>, // format: date-time

    #[serde(rename = "uploadExpiry", skip_serializing_if = "Option::is_none")]
    pub upload_expiry: Option<String>, // format: date-time

    #[serde(rename = "watermark", skip_serializing_if = "Option::is_none")]
    pub watermark: Option<Watermark>, // Watermark
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PublicDetails {
    title: String,
    share_link: String,
    channel_link: String,
    logo: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Input {
    #[serde(rename = "height")]
    pub height: Option<f32>,

    #[serde(rename = "width")]
    pub width: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Playback {
    #[serde(rename = "dash")]
    pub dash: Option<String>,

    #[serde(rename = "hls")]
    pub hls: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseInfo {
    pub code: u32,
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Watermark {
    created: Option<String>,
    #[serde(rename = "downloadedFrom")]
    downloaded_from: Option<String>,
    height: Option<f32>,
    name: Option<String>,
    opacity: Option<f32>,
    padding: Option<f64>,
    position: Option<String>,
    scale: Option<f32>,
    size: Option<f64>,
    uid: Option<String>,
    width: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WatermarkRequest {
    pub uid: Option<String>,
}

///
/// (maximum: 21600, minimum: 1)
///
/// The maximum duration in seconds for a video upload. Can be set for a video that is not yet uploaded...
///
///    allowedOrigins: Array<AllowedOrigins>OPTIONAL
///
///    Lists the origins allowed to display the video. Enter allowed origin domains in an array and use * ...
///
///   creator: stringOPTIONAL
///
///    (maxLength: 64)
///
///    A user-defined identifier for the media creator.
///
///   expiry: stringOPTIONAL
///
///    (format: date-time)
///
///    The date and time after upload when videos will not be accepted.
///
///   meta: unknownOPTIONAL
///
///    A user modifiable key-value store used to reference other systems of record for managing videos.
///
///   requireSignedURLs: booleanOPTIONAL
///
///    Indicates whether the video can be a accessed using the UID. When set to true, a signed token must ...
///
///   scheduledDeletion: stringOPTIONAL
///
///    (format: date-time)
///
///    Indicates the date and time at which the video will be deleted. Omit the field to indicate no chang...
///
///   thumbnailTimestampPct: numberOPTIONAL
///
///    (maximum: 1, minimum: 0)
///
///    The timestamp for a thumbnail image calculated as a percentage value of the video's duration. To co...

#[derive(Serialize, Deserialize, Default)]
pub struct DirectUploadRequestType {
    #[serde(rename = "maxDurationSeconds")]
    pub max_duration_seconds: u64,
    #[serde(rename = "allowedOrigins", skip_serializing_if = "Option::is_none")]
    pub allowed_origins: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, String>>,
    #[serde(rename = "requiredSignedURLs", skip_serializing_if = "Option::is_none")]
    pub required_signed_urls: Option<bool>,
    #[serde(rename = "scheduledDeletion")]
    pub scheduled_deletion: Option<String>,
    #[serde(
        rename = "thumbnailTimestampPct",
        skip_serializing_if = "Option::is_none"
    )]
    pub thumnail_timestamp_pct: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<WatermarkRequest>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DirectUploadResult {
    #[serde(rename = "scheduledDeletion")]
    pub scheduled_deletion: Option<String>,
    pub uid: Option<String>,
    #[serde(rename = "uploadURL")]
    pub upload_url: Option<String>,
    pub watermark: Option<Watermark>,
}

#[derive(Serialize, Deserialize)]
pub struct StreamResponseType<T> {
    pub errors: Vec<ResponseInfo>,
    pub messages: Option<Vec<ResponseInfo>>,
    pub success: bool,
    pub result: Option<T>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateDownloads {}

#[derive(Serialize, Deserialize)]
pub struct CreateDownloadResult {}
/// Delegated identity that can be serialized over the wire
#[derive(Serialize, Deserialize, Clone)]
pub struct DelegatedIdentityWire {
    /// raw bytes of delegated identity's public key
    pub from_key: Vec<u8>,
    /// JWK(JSON Web Key) encoded Secp256k1 secret key
    /// identity allowed to sign on behalf of `from_key`
    pub to_secret: JwkEcKey,
    /// Proof of delegation
    /// connecting from_key to `to_secret`
    pub delegation_chain: Vec<SignedDelegation>,
}

impl std::fmt::Debug for DelegatedIdentityWire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DelegatedIdentityWire").finish()
    }
}

impl TryFrom<DelegatedIdentityWire> for DelegatedIdentity {
    type Error = Box<dyn Error>;

    fn try_from(value: DelegatedIdentityWire) -> Result<DelegatedIdentity, Box<dyn Error>> {
        let to_secret = k256::SecretKey::from_jwk(&value.to_secret)?;
        let to_identity = Secp256k1Identity::from_private_key(to_secret);
        Self::new(
            value.from_key,
            Box::new(to_identity),
            value.delegation_chain,
        )
        .map_err(|e| Box::new(e) as Box<dyn Error>)
    }
}
