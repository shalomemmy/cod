use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Admin function to update system configuration
pub fn update_config(
    ctx: Context<UpdateConfig>,
    new_config: ReputationConfigUpdate,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let current_time = ReputationUtils::get_current_timestamp();

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Update configuration fields if provided
    if let Some(voting_cooldown) = new_config.voting_cooldown {
        require!(
            voting_cooldown >= 300 && voting_cooldown <= 86400,
            ReputationError::InvalidConfigurationValues
        );
        config.voting_cooldown = voting_cooldown;
    }

    if let Some(min_account_age) = new_config.min_account_age {
        require!(
            min_account_age >= 86400 && min_account_age <= 2592000,
            ReputationError::InvalidConfigurationValues
        );
        config.min_account_age = min_account_age;
    }

    if let Some(daily_vote_limit) = new_config.daily_vote_limit {
        require!(
            daily_vote_limit > 0 && daily_vote_limit <= 100,
            ReputationError::InvalidConfigurationValues
        );
        config.daily_vote_limit = daily_vote_limit;
    }

    if let Some(min_reputation_to_vote) = new_config.min_reputation_to_vote {
        require!(
            min_reputation_to_vote <= 10000,
            ReputationError::InvalidConfigurationValues
        );
        config.min_reputation_to_vote = min_reputation_to_vote;
    }

    if let Some(category_weights) = new_config.category_weights {
        ReputationUtils::validate_category_weights(&category_weights)?;
        config.category_weights = category_weights;
    }

    if let Some(role_thresholds) = new_config.role_thresholds {
        ReputationUtils::validate_role_thresholds(&role_thresholds)?;
        config.role_thresholds = role_thresholds;
    }

    if let Some(decay_rate) = new_config.decay_rate {
        require!(decay_rate <= 1000, ReputationError::InvalidConfigurationValues); // Max 10% per day
        config.decay_rate = decay_rate;
    }

    if let Some(decay_enabled) = new_config.decay_enabled {
        config.decay_enabled = decay_enabled;
    }

    config.last_updated = current_time;

    msg!("Configuration updated by admin: {}", ctx.accounts.admin.key());

    Ok(())
}

/// Get current system configuration
pub fn get_config(ctx: Context<GetConfig>) -> Result<ReputationConfigView> {
    let config = &ctx.accounts.config;

    let config_view = ReputationConfigView {
        admin: config.admin,
        voting_cooldown: config.voting_cooldown,
        min_account_age: config.min_account_age,
        daily_vote_limit: config.daily_vote_limit,
        min_reputation_to_vote: config.min_reputation_to_vote,
        category_weights: config.category_weights,
        role_thresholds: config.role_thresholds,
        current_season: config.current_season,
        season_start: config.season_start,
        season_duration: config.season_duration,
        total_users: config.total_users,
        decay_rate: config.decay_rate,
        decay_enabled: config.decay_enabled,
        initialized_at: config.initialized_at,
        last_updated: config.last_updated,
    };

    Ok(config_view)
}

/// Transfer admin authority to new admin
pub fn transfer_admin(
    ctx: Context<TransferAdmin>,
    new_admin: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Validate current admin authority
    require!(
        ctx.accounts.current_admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Update admin
    let old_admin = config.admin;
    config.admin = new_admin;
    config.last_updated = ReputationUtils::get_current_timestamp();

    msg!(
        "Admin authority transferred from {} to {}",
        old_admin,
        new_admin
    );

    Ok(())
}

/// Emergency pause/unpause system (admin only)
pub fn set_system_pause(
    ctx: Context<SetSystemPause>,
    paused: bool,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // For simplicity, we'll use the decay_enabled field to represent system pause
    // In a real implementation, you'd add a dedicated pause field
    config.decay_enabled = !paused;
    config.last_updated = ReputationUtils::get_current_timestamp();

    msg!(
        "System {} by admin: {}",
        if paused { "paused" } else { "unpaused" },
        ctx.accounts.admin.key()
    );

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ReputationConfigView {
    pub admin: Pubkey,
    pub voting_cooldown: u64,
    pub min_account_age: u64,
    pub daily_vote_limit: u8,
    pub min_reputation_to_vote: u64,
    pub category_weights: [u16; 4],
    pub role_thresholds: [u64; 5],
    pub current_season: u32,
    pub season_start: i64,
    pub season_duration: u64,
    pub total_users: u64,
    pub decay_rate: u16,
    pub decay_enabled: bool,
    pub initialized_at: i64,
    pub last_updated: i64,
}

#[derive(Accounts)]
#[instruction(new_config: ReputationConfigUpdate)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        constraint = admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct GetConfig<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,
}

#[derive(Accounts)]
#[instruction(new_admin: Pubkey)]
pub struct TransferAdmin<'info> {
    #[account(
        mut,
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        constraint = current_admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub current_admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(paused: bool)]
pub struct SetSystemPause<'info> {
    #[account(
        mut,
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        constraint = admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,
}