use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
};
use borsh::BorshSerialize;

use crate::error::SpotSurfError;
use crate::state::{SpotSurfInfo};


pub fn update_student_intro(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    review: String,
    rating: u16
) -> ProgramResult {

    msg!("Updating surfspot review...");
    msg!("Name: {}", name);
    msg!("Review: {}", review);
    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;

    msg!("unpacking state account");
    let mut account_data =
        try_from_slice_unchecked::<SpotSurfInfo>(&user_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    msg!("checking if movie account is initialized");
    if !account_data.is_initialized() {
        msg!("Account is not initialized");
        return Err(SpotSurfError::UninitializedAccount.into());
    }

    if user_account.owner != program_id {
        return Err(ProgramError::IllegalOwner);
    }

    let (pda, _bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref()], program_id);
    if pda != *user_account.key {
        msg!("Invalid seeds for PDA");
        return Err(SpotSurfError::InvalidPDA.into());
    }
    let update_len: usize = 1 + (4 + account_data.name.len()) + (4 + review.len());
    if update_len > 1000 {
        msg!("Data length is larger than 1000 bytes");
        return Err(SpotSurfError::InvalidDataLength.into());
    }
    if rating > 5 || rating < 1 {
        msg!("Invalid Rating");
        return Err(SpotSurfError::InvalidRating.into());
    }

    account_data.rate = rating;
    account_data.name = account_data.name;
    account_data.review = review;
    msg!("Rating: {}", account_data.rate);
    msg!("serializing account");
    account_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    Ok(())

}