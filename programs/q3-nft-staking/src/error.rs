use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Signer is not authorized to perform this action")]
    Unauthorized,
    #[msg("Account is not a valid Core asset")]
    InvalidAsset,
    #[msg("Account is not a valid Core collection for this program")]
    InvalidCollection,
    #[msg("Signer is not the owner of the asset")]
    NotOwner,
    #[msg("Asset is already staked")]
    AlreadyStaked,
    #[msg("Asset is not staked")]
    NotStaked,
    #[msg("Required attribute is missing or malformed")]
    AttributeMissing,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("No rewards accrued yet")]
    NothingToClaim,
    #[msg("Oracle state is already up to date")]
    OracleAlreadyUpToDate,
    #[msg("Oracle vault does not have enough lamports to pay the crank reward")]
    InsufficientVaultFunds,
}
