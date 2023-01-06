use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SpotSurfInfo {
    pub discriminator: String,
    pub is_init: bool,
    pub name: String,
    pub rate: u16,
    pub review: String,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct CommentCounter {
    pub discriminator: String,
    pub is_init: bool,
    pub counter: u8
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Comment {
    pub discriminator: String,
    pub is_init: bool,
    pub spot_info: Pubkey,
    pub comment: String
}

impl Sealed for SpotSurfInfo {}

impl IsInitialized for SpotSurfInfo {
    fn is_initialized(&self) -> bool {
        self.is_init
    }
}

impl IsInitialized for CommentCounter {
    fn is_initialized(&self) -> bool {
        self.is_init
    }
}

impl IsInitialized for Comment {
    fn is_initialized(&self) -> bool {
        self.is_init
    }
}