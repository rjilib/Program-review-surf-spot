use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpotSurfError {
    #[error("Account not initialized yet")]
    UninitializedAccount,
    #[error("PDA derived does not equal PDA passed in")]
    InvalidPDA,
    #[error("Input data exceeds max length")]
    InvalidDataLength,
    #[error("Accounts do not match")]
    IncorrectAccountError,
    #[error("Rating does not between 0 and 5")]
    InvalidRating,
}

impl From<SpotSurfError> for ProgramError {
    fn from(e: SpotSurfError) -> Self {
        ProgramError::Custom(e as u32)
    }
}