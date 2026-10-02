use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{Config, TransferOracle, CONFIG_SEED, ORACLE_SEED, ORACLE_VAULT_SEED};

#[derive(Accounts)]
pub struct CreateOracle<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(seeds = [CONFIG_SEED], bump = config.bump, has_one = admin)]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = admin,
        space = 8 + TransferOracle::INIT_SPACE,
        seeds = [ORACLE_SEED],
        bump
    )]
    pub oracle: Account<'info, TransferOracle>,
    #[account(mut, seeds = [ORACLE_VAULT_SEED], bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateOracle<'info> {
    pub fn handle_create_oracle(
        &mut self,
        vault_deposit: u64,
        bumps: &CreateOracleBumps,
    ) -> Result<()> {
        self.oracle.set_inner(TransferOracle {
            validation: TransferOracle::validation_at(Clock::get()?.unix_timestamp),
            bump: bumps.oracle,
            vault_bump: bumps.vault,
        });

        let ctx = CpiContext::new(
            self.system_program.key(),
            Transfer {
                from: self.admin.to_account_info(),
                to: self.vault.to_account_info(),
            },
        );
        transfer(ctx, vault_deposit)
    }
}
