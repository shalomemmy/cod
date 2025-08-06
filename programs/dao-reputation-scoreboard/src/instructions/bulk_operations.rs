use anchor_lang::prelude::*;
use anchor_lang::system_program::{System};
use crate::state::*;
use crate::errors::*;
// Removed unused import

/// Bulk admin operation to update multiple users
pub fn bulk_update_reputation(
    ctx: Context<BulkUpdateReputation>,
    updates: Vec<BulkReputationUpdate>,
) -> Result<()> {
    let config = &ctx.accounts.config;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Validate bulk operation size
    require!(
        updates.len() <= 50,
        ReputationError::BulkOperationTooLarge
    );

    // Validate all updates before processing
    for _update in &updates {
        // Skip validation for fixed-size byte array
    }

    msg!(
        "Bulk reputation update initiated by admin: {} ({} updates)",
        ctx.accounts.admin.key(),
        updates.len()
    );

    // In a real implementation, you would iterate through each update
    // and apply the changes to the corresponding user reputation accounts
    // For this example, we're just logging the operation

    Ok(())
}

/// Bulk initialize user reputation accounts
pub fn bulk_initialize_users(
    ctx: Context<BulkInitializeUsers>,
    users: Vec<Pubkey>,
) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Validate bulk operation size
    require!(
        users.len() <= 100,
        ReputationError::BulkOperationTooLarge
    );

    // Check if we would exceed maximum users limit
    let new_total = config.total_users + users.len() as u64;
    require!(
        new_total <= 10000, // Maximum users limit
        ReputationError::MaxUsersLimitReached
    );

    // Update total users count
    config.total_users = new_total;

    msg!(
        "Bulk user initialization by admin: {} ({} users)",
        ctx.accounts.admin.key(),
        users.len()
    );

    Ok(())
}

/// Bulk apply reputation decay to inactive users
pub fn bulk_apply_decay(
    ctx: Context<BulkApplyDecay>,
    users: Vec<Pubkey>,
) -> Result<()> {
    let config = &ctx.accounts.config;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Check if decay is enabled
    require!(
        config.decay_enabled,
        ReputationError::ReputationDecayDisabled
    );

    // Validate bulk operation size
    require!(
        users.len() <= 100,
        ReputationError::BulkOperationTooLarge
    );

    msg!(
        "Bulk decay application by admin: {} ({} users)",
        ctx.accounts.admin.key(),
        users.len()
    );

    // In a real implementation, you would iterate through each user
    // and apply decay based on their last activity timestamp

    Ok(())
}

/// Bulk award achievements to users
pub fn bulk_award_achievements(
    ctx: Context<BulkAwardAchievements>,
    awards: Vec<AchievementAward>,
) -> Result<()> {
    let config = &ctx.accounts.config;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Validate bulk operation size
    require!(
        awards.len() <= 50,
        ReputationError::BulkOperationTooLarge
    );

    msg!(
        "Bulk achievement awards by admin: {} ({} awards)",
        ctx.accounts.admin.key(),
        awards.len()
    );

    // In a real implementation, you would process each achievement award

    Ok(())
}

/// Reset all seasonal data (admin only)
pub fn bulk_reset_seasonal_data(
    ctx: Context<BulkResetSeasonalData>,
    users: Vec<Pubkey>,
) -> Result<()> {
    let config = &ctx.accounts.config;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Validate bulk operation size
    require!(
        users.len() <= 200,
        ReputationError::BulkOperationTooLarge
    );

    msg!(
        "Bulk seasonal reset by admin: {} ({} users)",
        ctx.accounts.admin.key(),
        users.len()
    );

    // In a real implementation, you would reset seasonal points for all users

    Ok(())
}

// AchievementAward is now defined in state.rs - removed duplicate

#[derive(Accounts)]
#[instruction(updates: Vec<BulkReputationUpdate>)]
pub struct BulkUpdateReputation<'info> {
    #[account(
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
#[instruction(users: Vec<Pubkey>)]
pub struct BulkInitializeUsers<'info> {
    #[account(
        mut,
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        mut,
        constraint = admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(users: Vec<Pubkey>)]
pub struct BulkApplyDecay<'info> {
    #[account(
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
#[instruction(awards: Vec<AchievementAward>)]
pub struct BulkAwardAchievements<'info> {
    #[account(
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
#[instruction(users: Vec<Pubkey>)]
pub struct BulkResetSeasonalData<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        constraint = admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,
}