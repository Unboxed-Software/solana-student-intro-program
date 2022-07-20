use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_pack::{IsInitialized, Sealed},
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct StudentInfo {
    pub discriminator: String,
    pub is_initialized: bool,
    pub name: String,
    pub msg: String,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ReplyCounter {
    pub discriminator: String,
    pub is_initialized: bool,
    pub counter: u8,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Reply {
    pub discriminator: String,
    pub is_initialized: bool,
    pub studentinfo: Pubkey,
    pub reply: String,
}

impl Sealed for StudentInfo {}

impl IsInitialized for StudentInfo {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for ReplyCounter {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl IsInitialized for Reply {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
