// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use serde::Serialize;
type Result<T> = std::result::Result<T, ic_agent::AgentError>;

#[derive(CandidType, Deserialize)]
pub enum KnownPrincipalType {
    CanisterIdUserIndex,
    CanisterIdPlatformOrchestrator,
    CanisterIdConfiguration,
    CanisterIdHotOrNotSubnetOrchestrator,
    CanisterIdProjectMemberIndex,
    CanisterIdTopicCacheIndex,
    CanisterIdRootCanister,
    CanisterIdDataBackup,
    CanisterIdSnsLedger,
    CanisterIdSnsWasm,
    CanisterIdPostCache,
    #[serde(rename = "CanisterIdSNSController")]
    CanisterIdSnsController,
    CanisterIdSnsGovernance,
    UserIdGlobalSuperAdmin,
}
#[derive(CandidType, Deserialize)]
pub struct IndividualUserTemplateInitArgs {
    pub known_principal_ids: Option<Vec<(KnownPrincipalType, Principal)>>,
    pub version: String,
    pub url_to_send_canister_metrics_to: Option<String>,
    pub profile_owner: Option<Principal>,
    pub upgrade_version_number: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub enum Result_ {
    Ok(bool),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum Result1 {
    Ok,
    Err(String),
}
#[derive(CandidType, Deserialize, Serialize, Clone)]
pub struct PostDetailsFromFrontend {
    pub is_nsfw: bool,
    pub hashtags: Vec<String>,
    pub description: String,
    pub video_uid: String,
    pub creator_consent_for_inclusion_in_hot_or_not: bool,
}
#[derive(CandidType, Deserialize)]
pub enum Result2 {
    Ok(u64),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum RejectionCode {
    NoError,
    CanisterError,
    SysTransient,
    DestinationInvalid,
    Unknown,
    SysFatal,
    CanisterReject,
}
#[derive(CandidType, Deserialize)]
pub enum TransferError {
    GenericError {
        message: String,
        error_code: candid::Nat,
    },
    TemporarilyUnavailable,
    BadBurn {
        min_burn_amount: candid::Nat,
    },
    Duplicate {
        duplicate_of: candid::Nat,
    },
    BadFee {
        expected_fee: candid::Nat,
    },
    CreatedInFuture {
        ledger_time: u64,
    },
    TooOld,
    InsufficientFunds {
        balance: candid::Nat,
    },
}
#[derive(CandidType, Deserialize)]
pub enum CdaoTokenError {
    NoBalance,
    InvalidRoot,
    CallError(RejectionCode, String),
    Transfer(TransferError),
    Unauthenticated,
}
#[derive(CandidType, Deserialize)]
pub enum Result3 {
    Ok(bool),
    Err(CdaoTokenError),
}
#[derive(CandidType, Deserialize)]
pub enum BetDirection {
    Hot,
    Not,
}
#[derive(CandidType, Deserialize)]
pub struct PlaceBetArg {
    pub bet_amount: u64,
    pub post_id: u64,
    pub bet_direction: BetDirection,
    pub post_canister_id: Principal,
}
#[derive(CandidType, Deserialize)]
pub struct SystemTime {
    pub nanos_since_epoch: u32,
    pub secs_since_epoch: u64,
}
#[derive(CandidType, Deserialize)]
pub enum BettingStatus {
    BettingOpen {
        number_of_participants: u8,
        ongoing_room: u64,
        ongoing_slot: u8,
        has_this_user_participated_in_this_post: Option<bool>,
        started_at: SystemTime,
    },
    BettingClosed,
}
#[derive(CandidType, Deserialize)]
pub enum BetOnCurrentlyViewingPostError {
    UserPrincipalNotSet,
    InsufficientBalance,
    UserAlreadyParticipatedInThisPost,
    BettingClosed,
    Unauthorized,
    PostCreatorCanisterCallFailed,
    UserNotLoggedIn,
}
#[derive(CandidType, Deserialize)]
pub enum Result4 {
    Ok(BettingStatus),
    Err(BetOnCurrentlyViewingPostError),
}
#[derive(CandidType, Deserialize)]
pub struct NamespaceForFrontend {
    pub id: u64,
    pub title: String,
    pub owner_id: Principal,
}
#[derive(CandidType, Deserialize)]
pub enum NamespaceErrors {
    UserNotSignedUp,
    ValueTooBig,
    NamespaceNotFound,
    Unauthorized,
}
#[derive(CandidType, Deserialize)]
pub enum Result5 {
    Ok(NamespaceForFrontend),
    Err(NamespaceErrors),
}
#[derive(CandidType, Deserialize)]
pub enum Result6 {
    Ok(Option<String>),
    Err(NamespaceErrors),
}
#[derive(CandidType, Deserialize)]
pub enum Result7 {
    Ok,
    Err(NamespaceErrors),
}
#[derive(CandidType, Deserialize)]
pub struct NeuronBasketConstructionParameters {
    pub dissolve_delay_interval_seconds: u64,
    pub count: u64,
}
#[derive(CandidType, Deserialize)]
pub struct Canister {
    pub id: Option<Principal>,
}
#[derive(CandidType, Deserialize)]
pub struct DappCanisters {
    pub canisters: Vec<Canister>,
}
#[derive(CandidType, Deserialize)]
pub struct LinearScalingCoefficient {
    pub slope_numerator: Option<u64>,
    pub intercept_icp_e8s: Option<u64>,
    pub from_direct_participation_icp_e8s: Option<u64>,
    pub slope_denominator: Option<u64>,
    pub to_direct_participation_icp_e8s: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub struct IdealMatchedParticipationFunction {
    pub serialized_representation: Option<String>,
}
#[derive(CandidType, Deserialize)]
pub struct NeuronsFundParticipationConstraints {
    pub coefficient_intervals: Vec<LinearScalingCoefficient>,
    pub max_neurons_fund_participation_icp_e8s: Option<u64>,
    pub min_direct_participation_threshold_icp_e8s: Option<u64>,
    pub ideal_matched_participation_function: Option<IdealMatchedParticipationFunction>,
}
#[derive(CandidType, Deserialize)]
pub struct TreasuryDistribution {
    pub total_e8s: u64,
}
#[derive(CandidType, Deserialize)]
pub struct NeuronDistribution {
    pub controller: Option<Principal>,
    pub dissolve_delay_seconds: u64,
    pub memo: u64,
    pub stake_e8s: u64,
    pub vesting_period_seconds: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub struct DeveloperDistribution {
    pub developer_neurons: Vec<NeuronDistribution>,
}
#[derive(CandidType, Deserialize)]
pub struct AirdropDistribution {
    pub airdrop_neurons: Vec<NeuronDistribution>,
}
#[derive(CandidType, Deserialize)]
pub struct SwapDistribution {
    pub total_e8s: u64,
    pub initial_swap_amount_e8s: u64,
}
#[derive(CandidType, Deserialize)]
pub struct FractionalDeveloperVotingPower {
    pub treasury_distribution: Option<TreasuryDistribution>,
    pub developer_distribution: Option<DeveloperDistribution>,
    pub airdrop_distribution: Option<AirdropDistribution>,
    pub swap_distribution: Option<SwapDistribution>,
}
#[derive(CandidType, Deserialize)]
pub enum InitialTokenDistribution {
    FractionalDeveloperVotingPower(FractionalDeveloperVotingPower),
}
#[derive(CandidType, Deserialize)]
pub struct Countries {
    pub iso_codes: Vec<String>,
}
#[derive(CandidType, Deserialize)]
pub struct SnsInitPayload {
    pub url: Option<String>,
    pub max_dissolve_delay_seconds: Option<u64>,
    pub max_dissolve_delay_bonus_percentage: Option<u64>,
    pub nns_proposal_id: Option<u64>,
    pub neurons_fund_participation: Option<bool>,
    pub min_participant_icp_e8s: Option<u64>,
    pub neuron_basket_construction_parameters: Option<NeuronBasketConstructionParameters>,
    pub fallback_controller_principal_ids: Vec<String>,
    pub token_symbol: Option<String>,
    pub final_reward_rate_basis_points: Option<u64>,
    pub max_icp_e8s: Option<u64>,
    pub neuron_minimum_stake_e8s: Option<u64>,
    pub confirmation_text: Option<String>,
    pub logo: Option<String>,
    pub name: Option<String>,
    pub swap_start_timestamp_seconds: Option<u64>,
    pub swap_due_timestamp_seconds: Option<u64>,
    pub initial_voting_period_seconds: Option<u64>,
    pub neuron_minimum_dissolve_delay_to_vote_seconds: Option<u64>,
    pub description: Option<String>,
    pub max_neuron_age_seconds_for_age_bonus: Option<u64>,
    pub min_participants: Option<u64>,
    pub initial_reward_rate_basis_points: Option<u64>,
    pub wait_for_quiet_deadline_increase_seconds: Option<u64>,
    pub transaction_fee_e8s: Option<u64>,
    pub dapp_canisters: Option<DappCanisters>,
    pub neurons_fund_participation_constraints: Option<NeuronsFundParticipationConstraints>,
    pub max_age_bonus_percentage: Option<u64>,
    pub initial_token_distribution: Option<InitialTokenDistribution>,
    pub reward_rate_transition_duration_seconds: Option<u64>,
    pub token_logo: Option<String>,
    pub token_name: Option<String>,
    pub max_participant_icp_e8s: Option<u64>,
    pub min_direct_participation_icp_e8s: Option<u64>,
    pub proposal_reject_cost_e8s: Option<u64>,
    pub restricted_countries: Option<Countries>,
    pub min_icp_e8s: Option<u64>,
    pub max_direct_participation_icp_e8s: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub enum ClaimStatus {
    Unclaimed,
    Claiming,
    Claimed,
}
#[derive(CandidType, Deserialize)]
pub struct AirdropInfo {
    pub principals_who_successfully_claimed: Vec<(Principal, ClaimStatus)>,
}
#[derive(CandidType, Deserialize)]
pub struct DeployedCdaoCanisters {
    pub airdrop_info: AirdropInfo,
    pub root: Principal,
    pub swap: Principal,
    pub ledger: Principal,
    pub index: Principal,
    pub governance: Principal,
}
#[derive(CandidType, Deserialize)]
pub enum CdaoDeployError {
    CycleError(String),
    Unregistered,
    CallError(RejectionCode, String),
    InvalidInitPayload(String),
    TokenLimit(u64),
    Unauthenticated,
}
#[derive(CandidType, Deserialize)]
pub enum Result8 {
    Ok(DeployedCdaoCanisters),
    Err(CdaoDeployError),
}
#[derive(CandidType, Deserialize)]
pub struct FolloweeArg {
    pub followee_canister_id: Principal,
    pub followee_principal_id: Principal,
}
#[derive(CandidType, Deserialize)]
pub enum FollowAnotherUserProfileError {
    UserITriedToFollowCrossCanisterCallFailed,
    UsersICanFollowListIsFull,
    Unauthorized,
    UserITriedToFollowHasTheirFollowersListFull,
    Unauthenticated,
}
#[derive(CandidType, Deserialize)]
pub enum Result9 {
    Ok(bool),
    Err(FollowAnotherUserProfileError),
}
#[derive(CandidType, Deserialize)]
pub enum BetMakerInformedStatus {
    InformedSuccessfully,
    Failed(String),
}
#[derive(CandidType, Deserialize)]
pub enum BetPayout {
    NotCalculatedYet,
    Calculated(u64),
}
#[derive(CandidType, Deserialize)]
pub struct BetDetails {
    pub bet_direction: BetDirection,
    pub bet_maker_canister_id: Principal,
    pub bet_maker_informed_status: Option<BetMakerInformedStatus>,
    pub amount: u64,
    pub payout: BetPayout,
}
#[derive(CandidType, Deserialize)]
pub enum Result10 {
    Ok(BetDetails),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub timestamp: u64,
}
#[derive(CandidType, Deserialize)]
pub enum PostStatus {
    BannedForExplicitness,
    BannedDueToUserReporting,
    Uploaded,
    CheckingExplicitness,
    ReadyToView,
    Transcoding,
    Deleted,
}
#[derive(CandidType, Deserialize)]
pub struct FeedScore {
    pub current_score: u64,
    pub last_synchronized_at: SystemTime,
    pub last_synchronized_score: u64,
}
#[derive(CandidType, Deserialize)]
pub struct PostViewStatistics {
    pub total_view_count: u64,
    pub average_watch_percentage: u8,
    pub threshold_view_count: u64,
}
#[derive(CandidType, Deserialize)]
pub struct AggregateStats {
    pub total_number_of_not_bets: u64,
    pub total_amount_bet: u64,
    pub total_number_of_hot_bets: u64,
}
#[derive(CandidType, Deserialize)]
pub enum RoomBetPossibleOutcomes {
    HotWon,
    BetOngoing,
    Draw,
    NotWon,
}
#[derive(CandidType, Deserialize)]
pub struct RoomDetails {
    pub total_hot_bets: u64,
    pub bets_made: Vec<(Principal, BetDetails)>,
    pub total_not_bets: u64,
    pub room_bets_total_pot: u64,
    pub bet_outcome: RoomBetPossibleOutcomes,
}
#[derive(CandidType, Deserialize)]
pub struct SlotDetails {
    pub room_details: Vec<(u64, RoomDetails)>,
}
#[derive(CandidType, Deserialize)]
pub struct HotOrNotDetails {
    pub hot_or_not_feed_score: FeedScore,
    pub aggregate_stats: AggregateStats,
    pub slot_history: Vec<(u8, SlotDetails)>,
}
#[derive(CandidType, Deserialize)]
pub struct Post {
    pub id: u64,
    pub is_nsfw: bool,
    pub status: PostStatus,
    pub share_count: u64,
    pub hashtags: Vec<String>,
    pub description: String,
    pub created_at: SystemTime,
    pub likes: Vec<Principal>,
    pub video_uid: String,
    pub home_feed_score: FeedScore,
    pub slots_left_to_be_computed: serde_bytes::ByteBuf,
    pub view_stats: PostViewStatistics,
    pub hot_or_not_details: Option<HotOrNotDetails>,
}
#[derive(CandidType, Deserialize)]
pub enum Result11 {
    Ok(Box<Post>),
    Err,
}
#[derive(CandidType, Deserialize)]
pub enum BetOutcomeForBetMaker {
    Won(u64),
    Draw(u64),
    Lost,
    AwaitingResult,
}
#[derive(CandidType, Deserialize)]
pub struct PlacedBetDetail {
    pub outcome_received: BetOutcomeForBetMaker,
    pub slot_id: u8,
    pub post_id: u64,
    pub room_id: u64,
    pub canister_id: Principal,
    pub bet_direction: BetDirection,
    pub amount_bet: u64,
    pub bet_placed_at: SystemTime,
}
#[derive(CandidType, Deserialize)]
pub struct PostDetailsForFrontend {
    pub id: u64,
    pub is_nsfw: bool,
    pub status: PostStatus,
    pub home_feed_ranking_score: u64,
    pub hashtags: Vec<String>,
    pub hot_or_not_betting_status: Option<BettingStatus>,
    pub like_count: u64,
    pub description: String,
    pub total_view_count: u64,
    pub created_by_display_name: Option<String>,
    pub created_at: SystemTime,
    pub created_by_unique_user_name: Option<String>,
    pub video_uid: String,
    pub created_by_user_principal_id: Principal,
    pub hot_or_not_feed_ranking_score: Option<u64>,
    pub liked_by_me: bool,
    pub created_by_profile_photo_url: Option<String>,
}
#[derive(CandidType, Deserialize)]
pub enum Result12 {
    Ok(SystemTime),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct MlFeedCacheItem {
    pub post_id: u64,
    pub canister_id: Principal,
    pub video_id: String,
    pub creator_principal_id: Option<Principal>,
}
#[derive(CandidType, Deserialize)]
pub enum GetPostsOfUserProfileError {
    ReachedEndOfItemsList,
    InvalidBoundsPassed,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}
#[derive(CandidType, Deserialize)]
pub enum Result13 {
    Ok(Vec<PostDetailsForFrontend>),
    Err(GetPostsOfUserProfileError),
}
#[derive(CandidType, Deserialize)]
pub struct FollowEntryDetail {
    pub canister_id: Principal,
    pub principal_id: Principal,
}
#[derive(CandidType, Deserialize)]
pub struct UserProfileGlobalStats {
    pub hot_bets_received: u64,
    pub not_bets_received: u64,
}
#[derive(CandidType, Deserialize)]
pub struct UserCanisterDetails {
    pub user_canister_id: Principal,
    pub profile_owner: Principal,
}
#[derive(CandidType, Deserialize)]
pub struct UserProfileDetailsForFrontend {
    pub unique_user_name: Option<String>,
    pub lifetime_earnings: u64,
    pub following_count: u64,
    pub profile_picture_url: Option<String>,
    pub display_name: Option<String>,
    pub principal_id: Principal,
    pub profile_stats: UserProfileGlobalStats,
    pub followers_count: u64,
    pub referrer_details: Option<UserCanisterDetails>,
}
#[derive(CandidType, Deserialize)]
pub enum MigrationInfo {
    MigratedFromHotOrNot { account_principal: Principal },
    NotMigrated,
    MigratedToYral { account_principal: Principal },
}
#[derive(CandidType, Deserialize)]
pub struct UserProfileDetailsForFrontendV2 {
    pub unique_user_name: Option<String>,
    pub lifetime_earnings: u64,
    pub migration_info: MigrationInfo,
    pub following_count: u64,
    pub profile_picture_url: Option<String>,
    pub display_name: Option<String>,
    pub principal_id: Principal,
    pub profile_stats: UserProfileGlobalStats,
    pub followers_count: u64,
    pub referrer_details: Option<UserCanisterDetails>,
}
#[derive(CandidType, Deserialize)]
pub enum SessionType {
    AnonymousSession,
    RegisteredSession,
}
#[derive(CandidType, Deserialize)]
pub enum Result14 {
    Ok(SessionType),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct SuccessHistoryItemV1 {
    pub post_id: u64,
    pub percentage_watched: f32,
    pub item_type: String,
    pub publisher_canister_id: Principal,
    pub cf_video_id: String,
    pub interacted_at: SystemTime,
}
#[derive(CandidType, Deserialize)]
pub enum Result15 {
    Ok(Vec<SuccessHistoryItemV1>),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum PaginationError {
    ReachedEndOfItemsList,
    InvalidBoundsPassed,
    ExceededMaxNumberOfItemsAllowedInOneRequest,
}
#[derive(CandidType, Deserialize)]
pub enum Result16 {
    Ok(Vec<Principal>),
    Err(PaginationError),
}
#[derive(CandidType, Deserialize)]
pub enum StakeEvent {
    BetOnHotOrNotPost(PlaceBetArg),
}
#[derive(CandidType, Deserialize)]
pub enum MintEvent {
    NewUserSignup {
        new_user_principal_id: Principal,
    },
    Referral {
        referrer_user_principal_id: Principal,
        referee_user_principal_id: Principal,
    },
}
#[derive(CandidType, Deserialize)]
pub enum HotOrNotOutcomePayoutEvent {
    WinningsEarnedFromBet {
        slot_id: u8,
        post_id: u64,
        room_id: u64,
        post_canister_id: Principal,
        winnings_amount: u64,
        event_outcome: BetOutcomeForBetMaker,
    },
    CommissionFromHotOrNotBet {
        slot_id: u8,
        post_id: u64,
        room_pot_total_amount: u64,
        room_id: u64,
        post_canister_id: Principal,
    },
}
#[derive(CandidType, Deserialize)]
pub enum TokenEvent {
    Stake {
        timestamp: SystemTime,
        details: StakeEvent,
        amount: u64,
    },
    Burn,
    Mint {
        timestamp: SystemTime,
        details: MintEvent,
        amount: u64,
    },
    Transfer {
        to_account: Principal,
        timestamp: SystemTime,
        amount: u64,
    },
    HotOrNotOutcomePayout {
        timestamp: SystemTime,
        details: HotOrNotOutcomePayoutEvent,
        amount: u64,
    },
    Receive {
        from_account: Principal,
        timestamp: SystemTime,
        amount: u64,
    },
}
#[derive(CandidType, Deserialize)]
pub enum Result17 {
    Ok(Vec<(u64, TokenEvent)>),
    Err(PaginationError),
}
#[derive(CandidType, Deserialize)]
pub struct WatchHistoryItem {
    pub post_id: u64,
    pub viewed_at: SystemTime,
    pub percentage_watched: f32,
    pub publisher_canister_id: Principal,
    pub cf_video_id: String,
}
#[derive(CandidType, Deserialize)]
pub enum Result18 {
    Ok(Vec<WatchHistoryItem>),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub body: serde_bytes::ByteBuf,
    pub headers: Vec<(String, String)>,
}
#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
    pub body: serde_bytes::ByteBuf,
    pub headers: Vec<(String, String)>,
    pub status_code: u16,
}
#[derive(CandidType, Deserialize)]
pub enum Result19 {
    Ok(Vec<String>),
    Err(NamespaceErrors),
}
#[derive(CandidType, Deserialize)]
pub enum Result20 {
    Ok(Vec<(u64, u8)>),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct BalanceInfo {
    pub balance: candid::Nat,
    pub withdrawable: candid::Nat,
    pub net_airdrop_reward: candid::Nat,
}
#[derive(CandidType, Deserialize)]
pub enum GameDirection {
    Dump,
    Pump,
}
#[derive(CandidType, Deserialize)]
pub struct ParticipatedGameInfo {
    pub game_direction: GameDirection,
    pub reward: candid::Nat,
    pub pumps: u64,
    pub dumps: u64,
    pub token_root: Principal,
}
#[derive(CandidType, Deserialize)]
pub enum Result21 {
    Ok(Vec<ParticipatedGameInfo>),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct PumpsAndDumps {
    pub pumps: candid::Nat,
    pub dumps: candid::Nat,
}
#[derive(CandidType, Deserialize)]
pub enum MigrationErrors {
    InvalidToCanister,
    InvalidFromCanister,
    MigrationInfoNotFound,
    UserNotRegistered,
    RequestCycleFromUserIndexFailed(String),
    UserIndexCanisterIdNotFound,
    Unauthorized,
    TransferToCanisterCallFailed(String),
    HotOrNotSubnetCanisterIdNotFound,
    AlreadyUsedForMigration,
    CanisterInfoFailed,
    AlreadyMigrated,
}
#[derive(CandidType, Deserialize)]
pub enum Result22 {
    Ok,
    Err(MigrationErrors),
}
#[derive(CandidType, Deserialize)]
pub enum PumpNDumpStateDiff {
    Participant(ParticipatedGameInfo),
    CreatorReward(candid::Nat),
}
#[derive(CandidType, Deserialize)]
pub enum AirdropError {
    NoBalance,
    CanisterPrincipalDoNotMatch,
    AlreadyClaimedAirdrop,
    RequestedAmountTooLow,
    InvalidRoot,
    CallError(RejectionCode, String),
    Transfer(TransferError),
}
#[derive(CandidType, Deserialize)]
pub enum Result23 {
    Ok,
    Err(AirdropError),
}
#[derive(CandidType, Deserialize)]
pub enum Result24 {
    Ok(String),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct IndividualUserCreatorDaoEntry {
    pub deployed_canisters: Vec<Principal>,
    pub individual_profile_id: Principal,
}
#[derive(CandidType, Deserialize)]
pub enum Result25 {
    Ok(IndividualUserCreatorDaoEntry),
    Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct Committed {
    pub total_direct_participation_icp_e8s: Option<u64>,
    pub total_neurons_fund_participation_icp_e8s: Option<u64>,
    pub sns_governance_canister_id: Option<Principal>,
}
#[derive(CandidType, Deserialize)]
pub enum Result26 {
    Committed(Committed),
    Aborted {},
}
#[derive(CandidType, Deserialize)]
pub struct SettleNeuronsFundParticipationRequest {
    pub result: Option<Result26>,
    pub nns_proposal_id: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub struct Principals {
    pub principals: Vec<Principal>,
}
#[derive(CandidType, Deserialize)]
pub struct NeuronsFundNeuron {
    pub controller: Option<Principal>,
    pub hotkeys: Option<Principals>,
    pub is_capped: Option<bool>,
    pub nns_neuron_id: Option<u64>,
    pub amount_icp_e8s: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub struct Ok {
    pub neurons_fund_neuron_portions: Vec<NeuronsFundNeuron>,
}
#[derive(CandidType, Deserialize)]
pub struct GovernanceError {
    pub error_message: String,
    pub error_type: i32,
}
#[derive(CandidType, Deserialize)]
pub enum Result27 {
    Ok(Ok),
    Err(GovernanceError),
}
#[derive(CandidType, Deserialize)]
pub struct SettleNeuronsFundParticipationResponse {
    pub result: Option<Result27>,
}
#[derive(CandidType, Deserialize)]
pub enum Result28 {
    Ok,
    Err(CdaoTokenError),
}
#[derive(CandidType, Deserialize)]
pub enum PostViewDetailsFromFrontend {
    WatchedMultipleTimes {
        percentage_watched: u8,
        watch_count: u8,
    },
    WatchedPartially {
        percentage_watched: u8,
    },
}
#[derive(CandidType, Deserialize)]
pub struct UserProfileUpdateDetailsFromFrontend {
    pub profile_picture_url: Option<String>,
    pub display_name: Option<String>,
}
#[derive(CandidType, Deserialize)]
pub enum UpdateProfileDetailsError {
    NotAuthorized,
}
#[derive(CandidType, Deserialize)]
pub enum Result29 {
    Ok(Box<UserProfileDetailsForFrontend>),
    Err(UpdateProfileDetailsError),
}
#[derive(CandidType, Deserialize)]
pub enum UpdateProfileSetUniqueUsernameError {
    UsernameAlreadyTaken,
    UserIndexCrossCanisterCallFailed,
    SendingCanisterDoesNotMatchUserCanisterId,
    NotAuthorized,
    UserCanisterEntryDoesNotExist,
}
#[derive(CandidType, Deserialize)]
pub enum Result30 {
    Ok,
    Err(UpdateProfileSetUniqueUsernameError),
}
#[derive(CandidType, Deserialize)]
pub struct FollowerArg {
    pub follower_canister_id: Principal,
    pub follower_principal_id: Principal,
}

#[derive(Clone)]
pub struct Service<'a>(pub Principal, pub &'a ic_agent::Agent);
impl Service<'_> {
    pub async fn add_device_id(&self, arg0: String) -> Result<Result_> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "add_device_id")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result_)?)
    }
    pub async fn add_dollr_to_liquidity_pool(
        &self,
        arg0: Principal,
        arg1: candid::Nat,
    ) -> Result<Result1> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "add_dollr_to_liquidity_pool")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result1)?)
    }
    pub async fn add_post_v_2(&self, arg0: PostDetailsFromFrontend) -> Result<Result2> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "add_post_v2")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result2)?)
    }
    pub async fn add_token(&self, arg0: Principal) -> Result<Result3> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "add_token")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result3)?)
    }
    pub async fn bet_on_currently_viewing_post(&self, arg0: PlaceBetArg) -> Result<Result4> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "bet_on_currently_viewing_post")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result4)?)
    }
    pub async fn check_and_update_scores_and_share_with_post_cache_if_difference_beyond_threshold(
        &self,
        arg0: Vec<u64>,
    ) -> Result<()> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(
                &self.0,
                "check_and_update_scores_and_share_with_post_cache_if_difference_beyond_threshold",
            )
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn clear_snapshot(&self) -> Result<()> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "clear_snapshot")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn create_a_namespace(&self, arg0: String) -> Result<Result5> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "create_a_namespace")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result5)?)
    }
    pub async fn delete_all_creator_token(&self) -> Result<()> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "delete_all_creator_token")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn delete_key_value_pair(&self, arg0: u64, arg1: String) -> Result<Result6> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "delete_key_value_pair")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result6)?)
    }
    pub async fn delete_multiple_key_value_pairs(
        &self,
        arg0: u64,
        arg1: Vec<String>,
    ) -> Result<Result7> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "delete_multiple_key_value_pairs")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result7)?)
    }
    pub async fn deploy_cdao_sns(&self, arg0: SnsInitPayload, arg1: u64) -> Result<Result8> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "deploy_cdao_sns")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result8)?)
    }
    pub async fn deployed_cdao_canisters(&self) -> Result<Vec<DeployedCdaoCanisters>> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "deployed_cdao_canisters")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<DeployedCdaoCanisters>)?)
    }
    pub async fn do_i_follow_this_user(&self, arg0: FolloweeArg) -> Result<Result9> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "do_i_follow_this_user")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result9)?)
    }
    pub async fn download_snapshot(&self, arg0: u64, arg1: u64) -> Result<serde_bytes::ByteBuf> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "download_snapshot")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, serde_bytes::ByteBuf)?)
    }
    pub async fn get_bet_details_for_a_user_on_a_post(
        &self,
        arg0: Principal,
        arg1: u64,
    ) -> Result<Result10> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "get_bet_details_for_a_user_on_a_post")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result10)?)
    }
    pub async fn get_device_identities(&self) -> Result<Vec<DeviceIdentity>> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_device_identities")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<DeviceIdentity>)?)
    }
    pub async fn get_entire_individual_post_detail_by_id(&self, arg0: u64) -> Result<Result11> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "get_entire_individual_post_detail_by_id")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result11)?)
    }
    pub async fn get_hot_or_not_bet_details_for_this_post(
        &self,
        arg0: u64,
    ) -> Result<BettingStatus> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "get_hot_or_not_bet_details_for_this_post")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, BettingStatus)?)
    }
    pub async fn get_hot_or_not_bets_placed_by_this_profile_with_pagination(
        &self,
        arg0: u64,
    ) -> Result<Vec<PlacedBetDetail>> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(
                &self.0,
                "get_hot_or_not_bets_placed_by_this_profile_with_pagination",
            )
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<PlacedBetDetail>)?)
    }
    pub async fn get_individual_hot_or_not_bet_placed_by_this_profile(
        &self,
        arg0: Principal,
        arg1: u64,
    ) -> Result<Option<PlacedBetDetail>> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(
                &self.0,
                "get_individual_hot_or_not_bet_placed_by_this_profile",
            )
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Option<PlacedBetDetail>)?)
    }
    pub async fn get_individual_post_details_by_id(
        &self,
        arg0: u64,
    ) -> Result<PostDetailsForFrontend> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "get_individual_post_details_by_id")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, PostDetailsForFrontend)?)
    }
    pub async fn get_last_access_time(&self) -> Result<Result12> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_last_access_time")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result12)?)
    }
    pub async fn get_last_canister_functionality_access_time(&self) -> Result<Result12> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_last_canister_functionality_access_time")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result12)?)
    }
    pub async fn get_ml_feed_cache_paginated(
        &self,
        arg0: u64,
        arg1: u64,
    ) -> Result<Vec<MlFeedCacheItem>> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "get_ml_feed_cache_paginated")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<MlFeedCacheItem>)?)
    }
    pub async fn get_posts_of_this_user_profile_with_pagination(
        &self,
        arg0: u64,
        arg1: u64,
    ) -> Result<Result13> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "get_posts_of_this_user_profile_with_pagination")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result13)?)
    }
    pub async fn get_posts_of_this_user_profile_with_pagination_cursor(
        &self,
        arg0: u64,
        arg1: u64,
    ) -> Result<Result13> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(
                &self.0,
                "get_posts_of_this_user_profile_with_pagination_cursor",
            )
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result13)?)
    }
    pub async fn get_principals_that_follow_this_profile_paginated(
        &self,
        arg0: Option<u64>,
    ) -> Result<Vec<(u64, FollowEntryDetail)>> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "get_principals_that_follow_this_profile_paginated")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<(u64, FollowEntryDetail,)>)?)
    }
    pub async fn get_principals_this_profile_follows_paginated(
        &self,
        arg0: Option<u64>,
    ) -> Result<Vec<(u64, FollowEntryDetail)>> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "get_principals_this_profile_follows_paginated")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<(u64, FollowEntryDetail,)>)?)
    }
    pub async fn get_profile_details(&self) -> Result<Box<UserProfileDetailsForFrontend>> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_profile_details")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Box<UserProfileDetailsForFrontend>)?)
    }
    pub async fn get_profile_details_v_2(&self) -> Result<UserProfileDetailsForFrontendV2> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_profile_details_v2")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, UserProfileDetailsForFrontendV2)?)
    }
    pub async fn get_rewarded_for_referral(&self, arg0: Principal, arg1: Principal) -> Result<()> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "get_rewarded_for_referral")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn get_rewarded_for_signing_up(&self) -> Result<()> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "get_rewarded_for_signing_up")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn get_session_type(&self) -> Result<Result14> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_session_type")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result14)?)
    }
    pub async fn get_stable_memory_size(&self) -> Result<u64> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_stable_memory_size")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, u64)?)
    }
    pub async fn get_success_history(&self) -> Result<Result15> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_success_history")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result15)?)
    }
    pub async fn get_token_roots_of_this_user_with_pagination_cursor(
        &self,
        arg0: u64,
        arg1: u64,
    ) -> Result<Result16> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(
                &self.0,
                "get_token_roots_of_this_user_with_pagination_cursor",
            )
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result16)?)
    }
    pub async fn get_user_caniser_cycle_balance(&self) -> Result<candid::Nat> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_user_caniser_cycle_balance")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, candid::Nat)?)
    }
    pub async fn get_user_propensity(&self) -> Result<f64> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_user_propensity")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, f64)?)
    }
    pub async fn get_user_utility_token_transaction_history_with_pagination(
        &self,
        arg0: u64,
        arg1: u64,
    ) -> Result<Result17> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(
                &self.0,
                "get_user_utility_token_transaction_history_with_pagination",
            )
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result17)?)
    }
    pub async fn get_utility_token_balance(&self) -> Result<u64> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_utility_token_balance")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, u64)?)
    }
    pub async fn get_version(&self) -> Result<String> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_version")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, String)?)
    }
    pub async fn get_version_number(&self) -> Result<u64> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_version_number")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, u64)?)
    }
    pub async fn get_watch_history(&self) -> Result<Result18> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "get_watch_history")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result18)?)
    }
    pub async fn get_well_known_principal_value(
        &self,
        arg0: KnownPrincipalType,
    ) -> Result<Option<Principal>> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "get_well_known_principal_value")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Option<Principal>)?)
    }
    pub async fn http_request(&self, arg0: HttpRequest) -> Result<HttpResponse> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "http_request")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, HttpResponse)?)
    }
    pub async fn list_namespace_keys(&self, arg0: u64) -> Result<Result19> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .query(&self.0, "list_namespace_keys")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result19)?)
    }
    pub async fn list_namespaces(&self, arg0: u64, arg1: u64) -> Result<Vec<NamespaceForFrontend>> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "list_namespaces")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Vec<NamespaceForFrontend>)?)
    }
    pub async fn load_snapshot(&self) -> Result<()> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "load_snapshot")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn net_earnings(&self) -> Result<candid::Nat> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "net_earnings")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, candid::Nat)?)
    }
    pub async fn once_reenqueue_timers_for_pending_bet_outcomes(&self) -> Result<Result20> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "once_reenqueue_timers_for_pending_bet_outcomes")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result20)?)
    }
    pub async fn pd_balance_info(&self) -> Result<BalanceInfo> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "pd_balance_info")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, BalanceInfo)?)
    }
    pub async fn played_game_count(&self) -> Result<u64> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "played_game_count")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, u64)?)
    }
    pub async fn played_game_info_with_pagination_cursor(
        &self,
        arg0: u64,
        arg1: u64,
    ) -> Result<Result21> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "played_game_info_with_pagination_cursor")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result21)?)
    }
    pub async fn pumps_and_dumps(&self) -> Result<PumpsAndDumps> {
        let args = Encode!()?;
        let bytes = self
            .1
            .query(&self.0, "pumps_and_dumps")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, PumpsAndDumps)?)
    }
    pub async fn read_key_value_pair(&self, arg0: u64, arg1: String) -> Result<Result6> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .query(&self.0, "read_key_value_pair")
            .with_arg(args)
            .call()
            .await?;
        Ok(Decode!(&bytes, Result6)?)
    }
    pub async fn receive_and_save_snaphot(
        &self,
        arg0: u64,
        arg1: serde_bytes::ByteBuf,
    ) -> Result<()> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "receive_and_save_snaphot")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn receive_bet_from_bet_makers_canister(
        &self,
        arg0: PlaceBetArg,
        arg1: Principal,
    ) -> Result<Result4> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "receive_bet_from_bet_makers_canister")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result4)?)
    }
    pub async fn receive_bet_winnings_when_distributed(
        &self,
        arg0: u64,
        arg1: BetOutcomeForBetMaker,
    ) -> Result<()> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "receive_bet_winnings_when_distributed")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn receive_data_from_hotornot(
        &self,
        arg0: Principal,
        arg1: u64,
        arg2: Vec<Post>,
    ) -> Result<Result22> {
        let args = Encode!(&arg0, &arg1, &arg2)?;
        let bytes = self
            .1
            .update(&self.0, "receive_data_from_hotornot")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result22)?)
    }
    pub async fn reconcile_user_state(&self, arg0: Vec<PumpNDumpStateDiff>) -> Result<Result1> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "reconcile_user_state")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result1)?)
    }
    pub async fn redeem_gdollr(&self, arg0: candid::Nat) -> Result<Result1> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "redeem_gdollr")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result1)?)
    }
    pub async fn request_airdrop(
        &self,
        arg0: Principal,
        arg1: Option<serde_bytes::ByteBuf>,
        arg2: candid::Nat,
        arg3: Principal,
    ) -> Result<Result23> {
        let args = Encode!(&arg0, &arg1, &arg2, &arg3)?;
        let bytes = self
            .1
            .update(&self.0, "request_airdrop")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result23)?)
    }
    pub async fn reset_ml_feed_cache(&self) -> Result<Result24> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "reset_ml_feed_cache")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result24)?)
    }
    pub async fn return_cycles_to_user_index_canister(
        &self,
        arg0: Option<candid::Nat>,
    ) -> Result<()> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "return_cycles_to_user_index_canister")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn save_snapshot_json(&self) -> Result<u32> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "save_snapshot_json")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, u32)?)
    }
    pub async fn send_creator_dao_stats_to_subnet_orchestrator(&self) -> Result<Result25> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "send_creator_dao_stats_to_subnet_orchestrator")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result25)?)
    }
    pub async fn set_controller_as_subnet_orchestrator(&self, arg0: Principal) -> Result<()> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "set_controller_as_subnet_orchestrator")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn settle_neurons_fund_participation(
        &self,
        arg0: SettleNeuronsFundParticipationRequest,
    ) -> Result<SettleNeuronsFundParticipationResponse> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "settle_neurons_fund_participation")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, SettleNeuronsFundParticipationResponse)?)
    }
    pub async fn stake_dollr_for_gdollr(&self, arg0: candid::Nat) -> Result<Result1> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "stake_dollr_for_gdollr")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result1)?)
    }
    pub async fn transfer_token_to_user_canister(
        &self,
        arg0: Principal,
        arg1: Principal,
        arg2: Option<serde_bytes::ByteBuf>,
        arg3: candid::Nat,
    ) -> Result<Result28> {
        let args = Encode!(&arg0, &arg1, &arg2, &arg3)?;
        let bytes = self
            .1
            .update(&self.0, "transfer_token_to_user_canister")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result28)?)
    }
    pub async fn transfer_tokens_and_posts(
        &self,
        arg0: Principal,
        arg1: Principal,
    ) -> Result<Result22> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "transfer_tokens_and_posts")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result22)?)
    }
    pub async fn update_last_access_time(&self) -> Result<Result24> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "update_last_access_time")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result24)?)
    }
    pub async fn update_last_canister_functionality_access_time(&self) -> Result<()> {
        let args = Encode!()?;
        let bytes = self
            .1
            .update(&self.0, "update_last_canister_functionality_access_time")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn update_ml_feed_cache(&self, arg0: Vec<MlFeedCacheItem>) -> Result<Result24> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_ml_feed_cache")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result24)?)
    }
    pub async fn update_post_add_view_details(
        &self,
        arg0: u64,
        arg1: PostViewDetailsFromFrontend,
    ) -> Result<()> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "update_post_add_view_details")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn update_post_as_ready_to_view(&self, arg0: u64) -> Result<()> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_post_as_ready_to_view")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn update_post_increment_share_count(&self, arg0: u64) -> Result<u64> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_post_increment_share_count")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, u64)?)
    }
    pub async fn update_post_status(&self, arg0: u64, arg1: PostStatus) -> Result<()> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "update_post_status")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn update_post_toggle_like_status_by_caller(&self, arg0: u64) -> Result<bool> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_post_toggle_like_status_by_caller")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, bool)?)
    }
    pub async fn update_profile_display_details(
        &self,
        arg0: UserProfileUpdateDetailsFromFrontend,
    ) -> Result<Result29> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_profile_display_details")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result29)?)
    }
    pub async fn update_profile_owner(&self, arg0: Option<Principal>) -> Result<Result1> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_profile_owner")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result1)?)
    }
    pub async fn update_profile_set_unique_username_once(&self, arg0: String) -> Result<Result30> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_profile_set_unique_username_once")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result30)?)
    }
    pub async fn update_profiles_i_follow_toggle_list_with_specified_profile(
        &self,
        arg0: FolloweeArg,
    ) -> Result<Result9> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(
                &self.0,
                "update_profiles_i_follow_toggle_list_with_specified_profile",
            )
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result9)?)
    }
    pub async fn update_profiles_that_follow_me_toggle_list_with_specified_profile(
        &self,
        arg0: FollowerArg,
    ) -> Result<Result9> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(
                &self.0,
                "update_profiles_that_follow_me_toggle_list_with_specified_profile",
            )
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result9)?)
    }
    pub async fn update_referrer_details(&self, arg0: UserCanisterDetails) -> Result<Result24> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_referrer_details")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result24)?)
    }
    pub async fn update_session_type(&self, arg0: SessionType) -> Result<Result24> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_session_type")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result24)?)
    }
    pub async fn update_success_history(&self, arg0: SuccessHistoryItemV1) -> Result<Result24> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_success_history")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result24)?)
    }
    pub async fn update_user_propensity(&self, arg0: f64) -> Result<Result24> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_user_propensity")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result24)?)
    }
    pub async fn update_watch_history(&self, arg0: WatchHistoryItem) -> Result<Result24> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "update_watch_history")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result24)?)
    }
    pub async fn update_well_known_principal(
        &self,
        arg0: KnownPrincipalType,
        arg1: Principal,
    ) -> Result<()> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "update_well_known_principal")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes)?)
    }
    pub async fn upgrade_creator_dao_governance_canisters(
        &self,
        arg0: serde_bytes::ByteBuf,
    ) -> Result<Result1> {
        let args = Encode!(&arg0)?;
        let bytes = self
            .1
            .update(&self.0, "upgrade_creator_dao_governance_canisters")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result1)?)
    }
    pub async fn write_key_value_pair(
        &self,
        arg0: u64,
        arg1: String,
        arg2: String,
    ) -> Result<Result6> {
        let args = Encode!(&arg0, &arg1, &arg2)?;
        let bytes = self
            .1
            .update(&self.0, "write_key_value_pair")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result6)?)
    }
    pub async fn write_multiple_key_value_pairs(
        &self,
        arg0: u64,
        arg1: Vec<(String, String)>,
    ) -> Result<Result7> {
        let args = Encode!(&arg0, &arg1)?;
        let bytes = self
            .1
            .update(&self.0, "write_multiple_key_value_pairs")
            .with_arg(args)
            .call_and_wait()
            .await?;
        Ok(Decode!(&bytes, Result7)?)
    }
}
