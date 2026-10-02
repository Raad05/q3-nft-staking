pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("DdcLnwhQQcBqtFp8e5xduGEcTGeHjiRVugJuEuoaEUWW");

#[program]
pub mod q3_nft_staking {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        rewards_per_day: u64,
        burn_bonus: u64,
    ) -> Result<()> {
        ctx.accounts
            .handle_initialize(rewards_per_day, burn_bonus, &ctx.bumps)
    }
    pub fn create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts.handle_create_collection(name, uri)
    }
    pub fn mint_nft(ctx: Context<MintNft>, name: String, uri: String) -> Result<()> {
        ctx.accounts.handle_mint_nft(name, uri, &ctx.bumps)
    }
    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.handle_stake(&ctx.bumps)
    }
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        ctx.accounts.handle_claim_rewards(&ctx.bumps)
    }
    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.handle_unstake(&ctx.bumps)
    }
    pub fn burn_staked_nft(ctx: Context<BurnStakedNft>) -> Result<()> {
        ctx.accounts.handle_burn_staked_nft(&ctx.bumps)
    }
    pub fn create_oracle(ctx: Context<CreateOracle>, vault_deposit: u64) -> Result<()> {
        ctx.accounts.handle_create_oracle(vault_deposit, &ctx.bumps)
    }
    pub fn add_oracle_plugin(ctx: Context<AddOraclePlugin>) -> Result<()> {
        ctx.accounts.handle_add_oracle_plugin(&ctx.bumps)
    }
    pub fn crank_oracle(ctx: Context<CrankOracle>) -> Result<()> {
        ctx.accounts.handle_crank_oracle()
    }
    pub fn transfer_nft(ctx: Context<TransferNft>) -> Result<()> {
        ctx.accounts.handle_transfer_nft()
    }
}
