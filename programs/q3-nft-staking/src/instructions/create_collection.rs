use anchor_lang::prelude::*;
use mpl_core::{
    instructions::CreateCollectionV2CpiBuilder,
    types::{Attribute, Attributes, Plugin, PluginAuthority, PluginAuthorityPair},
};

use crate::{Config, CONFIG_SEED, TOTAL_STAKED_KEY, UPDATE_AUTHORITY_SEED};

#[derive(Accounts)]
pub struct CreateCollection<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = admin)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub collection: Signer<'info>,
    /// CHECK: PDA that acts as the collection update authority and plugin authority
    #[account(seeds = [UPDATE_AUTHORITY_SEED], bump)]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: verified by address
    #[account(address = mpl_core::ID)]
    pub core_program: UncheckedAccount<'info>,
}

impl<'info> CreateCollection<'info> {
    pub fn handle_create_collection(&mut self, name: String, uri: String) -> Result<()> {
        CreateCollectionV2CpiBuilder::new(&self.core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .update_authority(Some(&self.update_authority.to_account_info()))
            .payer(&self.admin.to_account_info())
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .plugins(vec![PluginAuthorityPair {
                plugin: Plugin::Attributes(Attributes {
                    attribute_list: vec![Attribute {
                        key: TOTAL_STAKED_KEY.to_string(),
                        value: "0".to_string(),
                    }],
                }),
                authority: Some(PluginAuthority::UpdateAuthority),
            }])
            .invoke()?;
        Ok(())
    }
}
