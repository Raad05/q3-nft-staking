use anchor_lang::prelude::*;
use mpl_core::instructions::CreateV2CpiBuilder;

use crate::{Config, CONFIG_SEED, UPDATE_AUTHORITY_SEED};

#[derive(Accounts)]
pub struct MintNft<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = admin)]
    pub config: Account<'info, Config>,
    pub owner: SystemAccount<'info>,
    #[account(mut)]
    pub asset: Signer<'info>,
    /// CHECK: validated by the Core program
    #[account(mut, owner = mpl_core::ID)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: PDA that acts as the collection update authority
    #[account(seeds = [UPDATE_AUTHORITY_SEED], bump)]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: verified by address
    #[account(address = mpl_core::ID)]
    pub core_program: UncheckedAccount<'info>,
}

impl<'info> MintNft<'info> {
    pub fn handle_mint_nft(
        &mut self,
        name: String,
        uri: String,
        bumps: &MintNftBumps,
    ) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[UPDATE_AUTHORITY_SEED, &[bumps.update_authority]]];

        CreateV2CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .authority(Some(&self.update_authority.to_account_info()))
            .payer(&self.admin.to_account_info())
            .owner(Some(&self.owner.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .invoke_signed(signer_seeds)?;
        Ok(())
    }
}
