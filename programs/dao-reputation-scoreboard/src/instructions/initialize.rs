use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Initialize the reputation system with admin configuration
pub fn initialize_reputation_system(
    ctx: Context<InitializeReputationSystem>,
    voting_cooldown: u64,
    min_account_age: u64,
    daily_vote_limit: u8,
    min_reputation_to_vote: u64,
    category_weights: [u16; 4],
    role_thresholds: [u64; 5],
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let current_time = ReputationUtils::get_current_timestamp();

    // Validate configuration parameters
    ReputationUtils::validate_category_weights(&category_weights)?;
    ReputationUtils::validate_role_thresholds(&role_thresholds)?;
    
    // Validate other parameters
    require!(voting_cooldown >= 300, ReputationError::InvalidConfigurationValues); // Min 5 minutes
    require!(voting_cooldown <= 86400, ReputationError::InvalidConfigurationValues); // Max 24 hours
    require!(min_account_age >= 86400, ReputationError::InvalidConfigurationValues); // Min 1 day
    require!(min_account_age <= 2592000, ReputationError::InvalidConfigurationValues); // Max 30 days
    require!(daily_vote_limit > 0 && daily_vote_limit <= 100, ReputationError::InvalidConfigurationValues);
    require!(min_reputation_to_vote <= 10000, ReputationError::InvalidConfigurationValues);

    // Initialize configuration
    config.admin = ctx.accounts.admin.key();
    config.voting_cooldown = voting_cooldown;
    config.min_account_age = min_account_age;
    config.daily_vote_limit = daily_vote_limit;
    config.min_reputation_to_vote = min_reputation_to_vote;
    config.category_weights = category_weights;
    config.role_thresholds = role_thresholds;
    config.current_season = 1;
    config.season_start = current_time;
    config.season_duration = 2592000; // 30 days default
    config.total_users = 0;
    config.decay_rate = 10; // 0.1% per day default
    config.decay_enabled = true;
    config.initialized_at = current_time;
    config.last_updated = current_time;
    config.reserved = [0; 1];

    msg!("Reputation system initialized with admin: {}", ctx.accounts.admin.key());
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeReputationSystem<'info> {
    #[account(
        init,
        payer = admin,
        space = ReputationConfig::LEN,
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}