use candid::Principal;
// mxzaz-hqaaa-aaaar-qaada-cai
#[allow(unused)]
pub const CKBTC_LEDGER: Principal = Principal::from_slice(&[0, 0, 0, 0, 2, 48, 0, 6, 1, 1]);
// 500 Satoshis

pub const CKBTC_TREASURY_STORAGE_KEY: &str = "ckbtc-treasury-limit-v5";

// 1 million Satoshis
pub const SATS_CREDITED_STORAGE_KEY: &str = "sats-credited-limit-v0";
// 100,000 Satoshis
pub const SATS_DEDUCTED_STORAGE_KEY: &str = "sats-deducted-limit-v0";

pub const ADMIN_LOCAL_SECP_SK: [u8; 32] = [
    9, 64, 7, 55, 201, 208, 139, 219, 167, 201, 176, 6, 31, 109, 44, 248, 27, 241, 239, 56, 98,
    100, 158, 36, 79, 233, 172, 151, 228, 187, 8, 224,
];
// pub const LOCAL_METADATA_API_BASE: &str = "http://localhost:8001";

pub const SCHEMA_VERSION: u32 = 2;

// ckBTC transfer limits
pub const MAX_CKBTC_TRANSFER_SATS: u128 = 100_000;
