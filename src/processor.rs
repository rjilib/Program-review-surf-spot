use crate::instruction::{InstructionSpotSurf};
use crate::instructions;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult {
   let instruction =  InstructionSpotSurf::unpack(instruction_data).unwrap();
   match instruction {
        InstructionSpotSurf::InitSpotSurfInfo { name, rate, review } => {
            instructions::add_spot_surf(program_id, accounts, name, review, rate)
        }
        InstructionSpotSurf::UpdateSpotSurfInfo { name, rate, review } => {
            instructions::update_student_intro(program_id, accounts, name, review, rate)
        }
        InstructionSpotSurf::InitMint{} => {
            instructions::initialize_token_mint(program_id, accounts)
        }
        InstructionSpotSurf::AddComment { comment } => {
            instructions::add_comment(program_id, accounts, comment)
        } 
    }
}

