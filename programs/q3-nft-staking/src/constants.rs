use anchor_lang::prelude::*;

#[constant]
pub const CONFIG_SEED: &[u8] = b"config";

#[constant]
pub const REWARDS_MINT_SEED: &[u8] = b"rewards";

#[constant]
pub const UPDATE_AUTHORITY_SEED: &[u8] = b"update_authority";

#[constant]
pub const ORACLE_SEED: &[u8] = b"oracle";

#[constant]
pub const ORACLE_VAULT_SEED: &[u8] = b"oracle_vault";

#[constant]
pub const REWARDS_DECIMALS: u8 = 6;

#[constant]
pub const SECONDS_PER_DAY: i64 = 86_400;

#[constant]
pub const SECONDS_PER_HOUR: i64 = 3_600;

#[constant]
pub const OPEN_HOUR: i64 = 9;

#[constant]
pub const CLOSE_HOUR: i64 = 17;

#[constant]
pub const CRANK_WINDOW_SECS: i64 = 300;

#[constant]
pub const CRANK_REWARD_LAMPORTS: u64 = 10_000_000;

#[constant]
pub const STAKED_AT_KEY: &str = "staked_at";

#[constant]
pub const TOTAL_STAKED_KEY: &str = "total_staked";
