use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;
pub mod utils;

use instructions::*;
use state::*;
use errors::*;

// Replace this with the output from solana-keygen pubkey command
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// The main DAO Reputation Scoreboard program
#[program]
pub mod dao_reputation_scoreboard {
    use super::*;

    /// Initialize the reputation system with admin configuration
    pub fn initialize_reputation_system(
        ctx: Context<InitializeReputationSystem>,
        voting_cooldown: u64,
        min_account_age: u64,
        daily_vote_limit: u8,
        min_reputation_to_vote: u64,
        category_weights: [u16; 4], // [Governance, Development, Community, Treasury]
        role_thresholds: [u64; 5],  // Different role unlock thresholds
    ) -> Result<()> {
        instructions::initialize_reputation_system(
            ctx,
            voting_cooldown,
            min_account_age,
            daily_vote_limit,
            min_reputation_to_vote,
            category_weights,
            role_thresholds,
        )
    }

    // Rest of your code remains unchanged
    // ...
}