use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::try_from_slice_unchecked,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};
use borsh::BorshSerialize;

use spl_associated_token_account::get_associated_token_address;
use spl_token::{ID as TOKEN_PROGRAM_ID};
use std::convert::TryInto;

use crate::error::SpotSurfError;
use crate::state::{SpotSurfInfo, CommentCounter};

pub fn add_spot_surf(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    review: String,
    rating: u16,
) -> ProgramResult {
    msg!("Adding surf spot...");
    msg!("Name: {}", name);
    msg!("Review: {}", review);
    msg!("Rating: {}", rating);

    let account_info_iter = &mut accounts.iter();

    let initializer = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    let comment_counter = next_account_info(account_info_iter)?;
    let token_mint = next_account_info(account_info_iter)?;
    let mint_auth = next_account_info(account_info_iter)?;
    let user_ata = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    let (pda, bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref()], program_id);
    if pda != *user_account.key {
        msg!("Invalid seeds for PDA");
        return Err(SpotSurfError::InvalidPDA.into());
    }

    if rating > 5 || rating < 1 {
        msg!("Rating cannot be higher than 5");
        return Err(SpotSurfError::InvalidRating.into());
    }

    let spot_surf_descriminator = "spot_surf_info";

    let total_len: usize = (4 + spot_surf_descriminator.len()) + 1 + (4 + name.len()) + (4 + review.len());

    if total_len > 100 {
        msg!("Data length is larger than 1000 bytes");
        return Err(SpotSurfError::InvalidDataLength.into())
    }

    let account_len: usize = 1000;

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            user_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            user_account.clone(),
            system_program.clone(),
        ],
        &[&[initializer.key.as_ref(), &[bump_seed]]],
    )?;

    msg!("PDA created: {}", pda);
    msg!("unpacking state account");

    let mut account_data =
        try_from_slice_unchecked::<SpotSurfInfo>(&user_account.data.borrow()).unwrap();
    msg!("borrowed account data");

    msg!("checking if sportsurf account is already initialized");
    if account_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }


    account_data.name = name;
    account_data.review = review;
    account_data.is_init = true;
    account_data.rate = rating;
    msg!("serializing account");
    account_data.serialize(&mut &mut user_account.data.borrow_mut()[..])?;
    msg!("state account serialized");

    msg!("create comment counter");
    let counter_discriminator = "counter";
    let counter_len: usize = (4 + counter_discriminator.len()) + 1 + 1;

    let rent = Rent::get()?;
    let counter_rent_lamports = rent.minimum_balance(counter_len);

    let (counter, counter_bump) =
        Pubkey::find_program_address(&[pda.as_ref(), "reply".as_ref()], program_id);
    if counter != *comment_counter.key {
        msg!("Invalid seeds for PDA");
        return Err(ProgramError::InvalidArgument);
    }

    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            comment_counter.key,
            counter_rent_lamports,
            counter_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            comment_counter.clone(),
            system_program.clone(),
        ],
        &[&[pda.as_ref(), "reply".as_ref(), &[counter_bump]]],
    )?;
    msg!("reply counter created");

    let mut counter_data =
        try_from_slice_unchecked::<CommentCounter>(&comment_counter.data.borrow()).unwrap();

    msg!("checking if counter account is already initialized");
    if counter_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    counter_data.discriminator = counter_discriminator.to_string();
    counter_data.counter = 0;
    counter_data.is_init = true;
    msg!("reply count: {}", counter_data.counter);
    counter_data.serialize(&mut &mut comment_counter.data.borrow_mut()[..])?;



    // mint tokens here
    msg!("deriving mint authority");
    let (mint_pda, _mint_bump) = Pubkey::find_program_address(&[b"token_mint"], program_id);
    let (mint_auth_pda, mint_auth_bump) =
        Pubkey::find_program_address(&[b"token_auth"], program_id);

    if *token_mint.key != mint_pda {
        msg!("Incorrect token mint");
        return Err(SpotSurfError::IncorrectAccountError.into());
    }

    if *mint_auth.key != mint_auth_pda {
        msg!("Mint passed in and mint derived do not match");
        return Err(SpotSurfError::InvalidPDA.into());
    }

    if *user_ata.key != get_associated_token_address(initializer.key, token_mint.key) {
        msg!("Incorrect token mint");
        return Err(SpotSurfError::IncorrectAccountError.into());
    }

    if *token_program.key != TOKEN_PROGRAM_ID {
        msg!("Incorrect token program");
        return Err(SpotSurfError::IncorrectAccountError.into());
    }

    msg!("Minting 10 tokens to User associated token account");
    invoke_signed(
        // instruction
        &spl_token::instruction::mint_to(
            token_program.key,
            token_mint.key,
            user_ata.key,
            mint_auth.key,
            &[],
            10 * LAMPORTS_PER_SOL,
        )?,
        // account_infos
        &[token_mint.clone(), user_ata.clone(), mint_auth.clone()],
        // seeds
        &[&[b"token_auth", &[mint_auth_bump]]],
    )?;

    Ok(())
}