use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{
        BurnV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder,
    },
    types::{Attributes, FreezeDelegate, Plugin, PluginType, UpdateAuthority},
};

use crate::{
    error::ErrorCode, Config, CONFIG_SEED, REWARDS_MINT_SEED, STAKED_AT_KEY, TOTAL_STAKED_KEY,
    UPDATE_AUTHORITY_SEED,
};

#[derive(Accounts)]
pub struct BurnStakedNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump)]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        seeds = [REWARDS_MINT_SEED, config.key().as_ref()],
        bump = config.rewards_bump,
        mint::token_program = token_program,
    )]
    pub rewards_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = rewards_mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_rewards_ata: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: deserialized and validated in the handler
    #[account(mut, owner = mpl_core::ID)]
    pub asset: UncheckedAccount<'info>,
    /// CHECK: validated against the asset's update authority
    #[account(mut, owner = mpl_core::ID)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: PDA that acts as the collection update authority and plugin authority
    #[account(seeds = [UPDATE_AUTHORITY_SEED], bump)]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: verified by address
    #[account(address = mpl_core::ID)]
    pub core_program: UncheckedAccount<'info>,
}

impl<'info> BurnStakedNft<'info> {
    pub fn handle_burn_staked_nft(&mut self, bumps: &BurnStakedNftBumps) -> Result<()> {
        let asset = BaseAssetV1::from_bytes(&self.asset.data.borrow())
            .map_err(|_| ErrorCode::InvalidAsset)?;
        require_keys_eq!(asset.owner, self.user.key(), ErrorCode::NotOwner);
        require!(
            asset.update_authority == UpdateAuthority::Collection(self.collection.key()),
            ErrorCode::InvalidCollection
        );
        let (_, freeze, _) = fetch_plugin::<BaseAssetV1, FreezeDelegate>(
            &self.asset.to_account_info(),
            PluginType::FreezeDelegate,
        )
        .map_err(|_| ErrorCode::NotStaked)?;
        require!(freeze.frozen, ErrorCode::NotStaked);

        let (_, attributes, _) = fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        )
        .map_err(|_| ErrorCode::AttributeMissing)?;
        let staked_at: i64 = attributes
            .attribute_list
            .iter()
            .find(|a| a.key == STAKED_AT_KEY)
            .and_then(|a| a.value.parse().ok())
            .ok_or(ErrorCode::AttributeMissing)?;
        let rewards = self
            .config
            .calc_rewards(staked_at, Clock::get()?.unix_timestamp)?;
        let ua_seeds: &[&[&[u8]]] = &[&[UPDATE_AUTHORITY_SEED, &[bumps.update_authority]]];

        let bonus = rewards
            .checked_add(self.config.burn_bonus)
            .ok_or(ErrorCode::Overflow)?;
        self.mint_rewards(bonus)?;
        self.decrement_total_staked(ua_seeds)?;

        self.thaw(ua_seeds)?;
        BurnV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(Some(&self.system_program.to_account_info()))
            .invoke_signed(ua_seeds)?;
        Ok(())
    }

    fn mint_rewards(&self, amount: u64) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }
        let config_seeds: &[&[&[u8]]] = &[&[CONFIG_SEED, &[self.config.bump]]];
        let ctx = CpiContext::new_with_signer(
            self.token_program.key(),
            MintTo {
                mint: self.rewards_mint.to_account_info(),
                to: self.user_rewards_ata.to_account_info(),
                authority: self.config.to_account_info(),
            },
            config_seeds,
        );
        mint_to(ctx, amount)
    }

    fn decrement_total_staked(&self, signer_seeds: &[&[&[u8]]]) -> Result<()> {
        let (_, mut attributes, _) = fetch_plugin::<BaseCollectionV1, Attributes>(
            &self.collection.to_account_info(),
            PluginType::Attributes,
        )
        .map_err(|_| ErrorCode::AttributeMissing)?;
        let total = attributes
            .attribute_list
            .iter_mut()
            .find(|a| a.key == TOTAL_STAKED_KEY)
            .ok_or(ErrorCode::AttributeMissing)?;
        let count: u64 = total
            .value
            .parse()
            .map_err(|_| ErrorCode::AttributeMissing)?;
        total.value = count.checked_sub(1).ok_or(ErrorCode::Overflow)?.to_string();

        UpdateCollectionPluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(attributes))
            .invoke_signed(signer_seeds)?;
        Ok(())
    }

    fn thaw(&self, signer_seeds: &[&[&[u8]]]) -> Result<()> {
        UpdatePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(signer_seeds)?;
        Ok(())
    }
}
