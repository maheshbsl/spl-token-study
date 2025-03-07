pub mod instruction;
pub mod state;
pub mod processor;
pub mod error;

// Re-export if you want these to be accessible from the crate root
pub use instruction::*;
pub use state::*;
pub use processor::*;
pub use error::*;

// Program's entrypoint
use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

// Declare the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // This will be implemented in processor.rs
    processor::process_instruction(program_id, accounts, instruction_data)
}