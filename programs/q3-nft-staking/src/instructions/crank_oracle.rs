use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

use crate::{
    error::ErrorCode, TransferOracle, CLOSE_HOUR, CRANK_REWARD_LAMPORTS, CRANK_WINDOW_SECS,
    OPEN_HOUR, ORACLE_SEED, ORACLE_VAULT_SEED, SECONDS_PER_DAY, SECONDS_PER_HOUR,
};

#[derive(Accounts)]
pub struct CrankOracle<'info> {
    #[account(mut)]
    pub cranker: Signer<'info>,
    #[account(mut, seeds = [ORACLE_SEED], bump = oracle.bump)]
    pub oracle: Account<'info, TransferOracle>,
    #[account(mut, seeds = [ORACLE_VAULT_SEED], bump = oracle.vault_bump)]
    pub vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> CrankOracle<'info> {
    pub fn handle_crank_oracle(&mut self) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;
        let new_validation = TransferOracle::validation_at(now);
        require!(
            self.oracle.validation != new_validation,
            ErrorCode::OracleAlreadyUpToDate
        );
        self.oracle.validation = new_validation;

        let seconds_of_day = now.rem_euclid(SECONDS_PER_DAY);
        let near_boundary = [OPEN_HOUR, CLOSE_HOUR]
            .iter()
            .any(|hour| (seconds_of_day - hour * SECONDS_PER_HOUR).abs() <= CRANK_WINDOW_SECS);
        if !near_boundary {
            return Ok(());
        }

        let rent_floor = Rent::get()?.minimum_balance(0);
        require!(
            self.vault.lamports() >= CRANK_REWARD_LAMPORTS + rent_floor,
            ErrorCode::InsufficientVaultFunds
        );
        let signer_seeds: &[&[&[u8]]] = &[&[ORACLE_VAULT_SEED, &[self.oracle.vault_bump]]];
        let ctx = CpiContext::new_with_signer(
            self.system_program.key(),
            Transfer {
                from: self.vault.to_account_info(),
                to: self.cranker.to_account_info(),
            },
            signer_seeds,
        );
        transfer(ctx, CRANK_REWARD_LAMPORTS)
    }
}
