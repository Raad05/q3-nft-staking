use anchor_lang::prelude::*;
use mpl_core::instructions::TransferV1CpiBuilder;

use crate::{TransferOracle, ORACLE_SEED};

#[derive(Accounts)]
pub struct TransferNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub new_owner: SystemAccount<'info>,
    /// CHECK: validated by the Core program (owner must be the signer)
    #[account(mut, owner = mpl_core::ID)]
    pub asset: UncheckedAccount<'info>,
    /// CHECK: validated by the Core program against the asset
    #[account(owner = mpl_core::ID)]
    pub collection: UncheckedAccount<'info>,
    #[account(seeds = [ORACLE_SEED], bump = oracle.bump)]
    pub oracle: Account<'info, TransferOracle>,
    pub system_program: Program<'info, System>,
    /// CHECK: verified by address
    #[account(address = mpl_core::ID)]
    pub core_program: UncheckedAccount<'info>,
}

impl<'info> TransferNft<'info> {
    pub fn handle_transfer_nft(&mut self) -> Result<()> {
        TransferV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.owner.to_account_info())
            .authority(Some(&self.owner.to_account_info()))
            .new_owner(&self.new_owner.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            .add_remaining_account(&self.oracle.to_account_info(), false, false)
            .invoke()?;
        Ok(())
    }
}
