use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Admin function to manually update user reputation
pub fn update_user_reputation(
    ctx: Context<UpdateUserReputation>,
    category: ReputationCategory,
    points_change: i64,
    reason: String,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_reputation = &mut ctx.accounts.user_reputation;
    let current_time = ReputationUtils::get_current_timestamp();

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Validate reason string length
    ReputationUtils::validate_string_length(&reason, 200)?;

    let category_index = category.to_index();

    // Apply points change
    if points_change >= 0 {
        user_reputation.category_points[category_index] = ReputationUtils::safe_add_points(
            user_reputation.category_points[category_index],
            points_change as u64,
        )?;
        user_reputation.raw_votes[category_index] = ReputationUtils::safe_add_points(
            user_reputation.raw_votes[category_index],
            points_change as u64,
        )?;
    } else {
        let points_to_subtract = (-points_change) as u64;
        user_reputation.category_points[category_index] = ReputationUtils::safe_subtract_points(
            user_reputation.category_points[category_index],
            points_to_subtract,
        )?;
    }

    // Recalculate total score
    user_reputation.calculate_total_score(&config.category_weights);

    // Update role level
    user_reputation.role_level = ReputationUtils::calculate_role_level(
        user_reputation.total_score,
        &config.role_thresholds,
    );

    // Update timestamps
    user_reputation.last_updated = current_time;
    user_reputation.last_activity = current_time;

    msg!(
        "Admin {} updated reputation for user {} in category {:?}: {} points. Reason: {}",
        ctx.accounts.admin.key(),
        user_reputation.user,
        category,
        points_change,
        reason
    );

    Ok(())
}

/// Initialize a new user reputation account
pub fn initialize_user_reputation(ctx: Context<InitializeUserReputation>) -> Result<()> {
    let user_reputation = &mut ctx.accounts.user_reputation;
    let config = &mut ctx.accounts.config;
    let current_time = ReputationUtils::get_current_timestamp();

    // Initialize user reputation
    user_reputation.user = ctx.accounts.user.key();
    user_reputation.category_points = [0; 4];
    user_reputation.total_score = 0;
    user_reputation.raw_votes = [0; 4];
    user_reputation.role_level = 0;
    user_reputation.achievements = 0;
    user_reputation.current_streak = 0;
    user_reputation.longest_streak = 0;
    user_reputation.last_activity = current_time;
    user_reputation.created_at = current_time;
    user_reputation.last_updated = current_time;
    user_reputation.seasonal_points = [0; 4];
    user_reputation.best_season_rank = 0;
    user_reputation.votes_cast = 0;
    user_reputation.reserved = [0; 16];
    user_reputation.reserved = [0; 4];

    // Increment total users count
    config.total_users += 1;

    msg!("User reputation initialized for: {}", ctx.accounts.user.key());

    Ok(())
}

#[derive(Accounts)]
#[instruction(category: ReputationCategory, points_change: i64, reason: String)]
pub struct UpdateUserReputation<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        mut,
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    #[account(
        constraint = admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// CHECK: User account being updated
    pub user: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct InitializeUserReputation<'info> {
    #[account(
        mut,
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        init,
        payer = user,
        space = UserReputation::LEN,
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
}