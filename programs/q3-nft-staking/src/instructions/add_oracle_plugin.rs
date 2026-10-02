use anchor_lang::prelude::*;
use mpl_core::{
    instructions::AddCollectionExternalPluginAdapterV1CpiBuilder,
    types::{
        ExternalPluginAdapterInitInfo, HookableLifecycleEvent, OracleInitInfo, PluginAuthority,
        ValidationResultsOffset,
    },
    ExternalCheckResultBits,
};

use crate::{Config, TransferOracle, CONFIG_SEED, ORACLE_SEED, UPDATE_AUTHORITY_SEED};

#[derive(Accounts)]
pub struct AddOraclePlugin<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = admin)]
    pub config: Account<'info, Config>,
    #[account(seeds = [ORACLE_SEED], bump = oracle.bump)]
    pub oracle: Account<'info, TransferOracle>,
    /// CHECK: validated by the Core program (update authority must be our PDA)
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

impl<'info> AddOraclePlugin<'info> {
    pub fn handle_add_oracle_plugin(&mut self, bumps: &AddOraclePluginBumps) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[UPDATE_AUTHORITY_SEED, &[bumps.update_authority]]];

        let reject_only = ExternalCheckResultBits::new().with_can_reject(true).into();

        AddCollectionExternalPluginAdapterV1CpiBuilder::new(&self.core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.admin.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .init_info(ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
                base_address: self.oracle.key(),
                init_plugin_authority: Some(PluginAuthority::UpdateAuthority),
                lifecycle_checks: vec![(HookableLifecycleEvent::Transfer, reject_only)],
                base_address_config: None,
                results_offset: Some(ValidationResultsOffset::Anchor),
            }))
            .invoke_signed(signer_seeds)?;
        Ok(())
    }
}
