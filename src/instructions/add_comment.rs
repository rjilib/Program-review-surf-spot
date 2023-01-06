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
use crate::state::{CommentCounter, Comment};


pub fn add_comment(
    program_id: &Pubkey, 
    accounts: &[AccountInfo], 
    comment: String
) -> ProgramResult {
    msg!("Adding Comment...");
    msg!("Comment: {}", comment);

    let account_info_iter = &mut accounts.iter();

    let commenter = next_account_info(account_info_iter)?;
    let user_account = next_account_info(account_info_iter)?;
    let comment_counter = next_account_info(account_info_iter)?;
    let comment_account = next_account_info(account_info_iter)?;
    let token_mint = next_account_info(account_info_iter)?;
    let mint_auth = next_account_info(account_info_iter)?;
    let user_ata = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;

    let mut counter_data =
        try_from_slice_unchecked::<CommentCounter>(&comment_counter.data.borrow()).unwrap();

    let comment_discriminator = "reply";
    let account_len: usize = (4 + comment_discriminator.len()) + 1 + 32 + (4 + comment.len());

    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[
            user_account.key.as_ref(),
            counter_data.counter.to_be_bytes().as_ref(),
        ],
        program_id,
    );
    if pda != *comment_account.key {
        msg!("Invalid seeds for PDA");
        return Err(SpotSurfError::InvalidPDA.into());
    }

    invoke_signed(
        &system_instruction::create_account(
            commenter.key,
            comment_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            commenter.clone(),
            comment_account.clone(),
            system_program.clone(),
        ],
        &[&[
            user_account.key.as_ref(),
            counter_data.counter.to_be_bytes().as_ref(),
            &[bump_seed],
        ]],
    )?;

    msg!("Created Reply Account");
    let mut comment_data = try_from_slice_unchecked::<Comment>(&comment_account.data.borrow()).unwrap();

    msg!("checking if comment account is already initialized");
    if comment_data.is_initialized() {
        msg!("Account already initialized");
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    comment_data.discriminator = comment_discriminator.to_string();
    comment_data.spot_info = *user_account.key;
    comment_data.comment = comment;
    comment_data.is_init = true;
    comment_data.serialize(&mut &mut comment_account.data.borrow_mut()[..])?;
    msg!("Comment Count: {}", counter_data.counter);
    counter_data.counter += 1;
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

    if *user_ata.key != get_associated_token_address(commenter.key, token_mint.key) {
        msg!("Incorrect token mint");
        return Err(SpotSurfError::IncorrectAccountError.into());
    }

    if *token_program.key != TOKEN_PROGRAM_ID {
        msg!("Incorrect token program");
        return Err(SpotSurfError::IncorrectAccountError.into());
    }

    msg!("Minting 5 tokens to User associated token account");
    invoke_signed(
        // instruction
        &spl_token::instruction::mint_to(
            token_program.key,
            token_mint.key,
            user_ata.key,
            mint_auth.key,
            &[],
            5 * LAMPORTS_PER_SOL,
        )?,
        // account_infos
        &[token_mint.clone(), user_ata.clone(), mint_auth.clone()],
        // seeds
        &[&[b"token_auth", &[mint_auth_bump]]],
    )?;

    Ok(())
}