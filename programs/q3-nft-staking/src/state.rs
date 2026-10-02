use anchor_lang::prelude::*;

use crate::{error::ErrorCode, SECONDS_PER_DAY};

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub rewards_per_day: u64,
    pub burn_bonus: u64,
    pub rewards_bump: u8,
    pub bump: u8,
}

impl Config {
    pub fn calc_rewards(&self, staked_at: i64, now: i64) -> Result<u64> {
        let elapsed = now
            .checked_sub(staked_at)
            .ok_or(ErrorCode::Overflow)?
            .max(0) as u128;
        let rewards = elapsed
            .checked_mul(self.rewards_per_day as u128)
            .ok_or(ErrorCode::Overflow)?
            / SECONDS_PER_DAY as u128;
        u64::try_from(rewards).map_err(|_| ErrorCode::Overflow.into())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum ExternalValidationResult {
    Approved,
    Rejected,
    Pass,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace)]
pub enum OracleValidation {
    Uninitialized,
    V1 {
        create: ExternalValidationResult,
        transfer: ExternalValidationResult,
        burn: ExternalValidationResult,
        update: ExternalValidationResult,
    },
}

#[account]
#[derive(InitSpace)]
pub struct TransferOracle {
    pub validation: OracleValidation,
    pub bump: u8,
    pub vault_bump: u8,
}

impl TransferOracle {
    pub fn transfer_result_at(now: i64) -> ExternalValidationResult {
        let hour = now.rem_euclid(SECONDS_PER_DAY) / crate::SECONDS_PER_HOUR;
        if (crate::OPEN_HOUR..crate::CLOSE_HOUR).contains(&hour) {
            ExternalValidationResult::Pass
        } else {
            ExternalValidationResult::Rejected
        }
    }

    pub fn validation_at(now: i64) -> OracleValidation {
        OracleValidation::V1 {
            create: ExternalValidationResult::Pass,
            transfer: Self::transfer_result_at(now),
            burn: ExternalValidationResult::Pass,
            update: ExternalValidationResult::Pass,
        }
    }
}
