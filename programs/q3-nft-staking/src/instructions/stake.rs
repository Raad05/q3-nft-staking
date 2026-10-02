use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{
        AddPluginV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder,
    },
    types::{
        Attribute, Attributes, BurnDelegate, FreezeDelegate, Plugin, PluginAuthority, PluginType,
        UpdateAuthority,
    },
};

use crate::{error::ErrorCode, STAKED_AT_KEY, TOTAL_STAKED_KEY, UPDATE_AUTHORITY_SEED};

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: deserialized and validated in the handler
    #[account(mut, owner = mpl_core::ID)]
    pub asset: UncheckedAccount<'info>,
    /// CHECK: deserialized and validated in the handler
    #[account(mut, owner = mpl_core::ID)]
    pub collection: UncheckedAccount<'info>,
    /// CHECK: PDA that acts as the collection update authority and plugin authority
    #[account(seeds = [UPDATE_AUTHORITY_SEED], bump)]
    pub update_authority: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: verified by address
    #[account(address = mpl_core::ID)]
    pub core_program: UncheckedAccount<'info>,
}

impl<'info> Stake<'info> {
    pub fn handle_stake(&mut self, bumps: &StakeBumps) -> Result<()> {
        let asset = BaseAssetV1::from_bytes(&self.asset.data.borrow())
            .map_err(|_| ErrorCode::InvalidAsset)?;
        require_keys_eq!(asset.owner, self.user.key(), ErrorCode::NotOwner);
        require!(
            asset.update_authority == UpdateAuthority::Collection(self.collection.key()),
            ErrorCode::InvalidCollection
        );
        require!(
            fetch_plugin::<BaseAssetV1, FreezeDelegate>(
                &self.asset.to_account_info(),
                PluginType::FreezeDelegate
            )
            .is_err(),
            ErrorCode::AlreadyStaked
        );

        let signer_seeds: &[&[&[u8]]] = &[&[UPDATE_AUTHORITY_SEED, &[bumps.update_authority]]];
        let delegate = PluginAuthority::Address {
            address: self.update_authority.key(),
        };

        AddPluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.user.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::BurnDelegate(BurnDelegate {}))
            .init_authority(delegate.clone())
            .invoke()?;

        AddPluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.user.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
            .init_authority(delegate)
            .invoke()?;

        let now = Clock::get()?.unix_timestamp.to_string();
        match fetch_plugin::<BaseAssetV1, Attributes>(
            &self.asset.to_account_info(),
            PluginType::Attributes,
        ) {
            Ok((_, mut attributes, _)) => {
                match attributes
                    .attribute_list
                    .iter_mut()
                    .find(|a| a.key == STAKED_AT_KEY)
                {
                    Some(attribute) => attribute.value = now,
                    None => attributes.attribute_list.push(Attribute {
                        key: STAKED_AT_KEY.to_string(),
                        value: now,
                    }),
                }
                UpdatePluginV1CpiBuilder::new(&self.core_program.to_account_info())
                    .asset(&self.asset.to_account_info())
                    .collection(Some(&self.collection.to_account_info()))
                    .payer(&self.user.to_account_info())
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::Attributes(attributes))
                    .invoke_signed(signer_seeds)?;
            }
            Err(_) => {
                AddPluginV1CpiBuilder::new(&self.core_program.to_account_info())
                    .asset(&self.asset.to_account_info())
                    .collection(Some(&self.collection.to_account_info()))
                    .payer(&self.user.to_account_info())
                    .authority(Some(&self.update_authority.to_account_info()))
                    .system_program(&self.system_program.to_account_info())
                    .plugin(Plugin::Attributes(Attributes {
                        attribute_list: vec![Attribute {
                            key: STAKED_AT_KEY.to_string(),
                            value: now,
                        }],
                    }))
                    .init_authority(PluginAuthority::UpdateAuthority)
                    .invoke_signed(signer_seeds)?;
            }
        }

        self.update_total_staked(signer_seeds)
    }

    fn update_total_staked(&self, signer_seeds: &[&[&[u8]]]) -> Result<()> {
        let collection = BaseCollectionV1::from_bytes(&self.collection.data.borrow())
            .map_err(|_| ErrorCode::InvalidCollection)?;
        require_keys_eq!(
            collection.update_authority,
            self.update_authority.key(),
            ErrorCode::InvalidCollection
        );

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
        total.value = count.checked_add(1).ok_or(ErrorCode::Overflow)?.to_string();

        UpdateCollectionPluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(attributes))
            .invoke_signed(signer_seeds)?;
        Ok(())
    }
}
