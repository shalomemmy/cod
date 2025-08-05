use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Allow users to claim role unlocks based on reputation thresholds
pub fn claim_role_unlock(
    ctx: Context<ClaimRoleUnlock>,
    role_level: u8,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_reputation = &mut ctx.accounts.user_reputation;

    // Validate role level
    require!(role_level > 0 && role_level <= 5, ReputationError::InvalidRoleLevel);

    // Check if user meets the threshold for this role
    let threshold_index = (role_level - 1) as usize;
    let required_score = config.role_thresholds[threshold_index];

    require!(
        user_reputation.total_score >= required_score,
        ReputationError::RoleUnlockRequirementsNotMet
    );

    // Check if user already has this role or higher
    require!(
        user_reputation.role_level < role_level,
        ReputationError::RoleUnlockRequirementsNotMet
    );

    // Update user's role level
    user_reputation.role_level = role_level;
    user_reputation.last_updated = ReputationUtils::get_current_timestamp();

    // Award role achievement if it's a significant milestone
    match role_level {
        3 => user_reputation.award_achievement(AchievementType::TopContributor),
        5 => user_reputation.award_achievement(AchievementType::CommunityBuilder),
        _ => {}
    }

    msg!(
        "User {} claimed role level {} with score {}",
        user_reputation.user,
        role_level,
        user_reputation.total_score
    );

    Ok(())
}

/// Check role unlock requirements for a user
pub fn check_role_requirements(
    ctx: Context<CheckRoleRequirements>,
    role_level: u8,
) -> Result<bool> {
    let config = &ctx.accounts.config;
    let user_reputation = &ctx.accounts.user_reputation;

    // Validate role level
    require!(role_level > 0 && role_level <= 5, ReputationError::InvalidRoleLevel);

    let threshold_index = (role_level - 1) as usize;
    let required_score = config.role_thresholds[threshold_index];

    let meets_requirements = user_reputation.total_score >= required_score 
        && user_reputation.role_level < role_level;

    msg!(
        "User {} role {} requirements check: {} (score: {}, required: {})",
        user_reputation.user,
        role_level,
        meets_requirements,
        user_reputation.total_score,
        required_score
    );

    Ok(meets_requirements)
}

/// Get all available role unlocks for a user
pub fn get_available_role_unlocks(
    ctx: Context<GetAvailableRoleUnlocks>,
) -> Result<Vec<u8>> {
    let config = &ctx.accounts.config;
    let user_reputation = &ctx.accounts.user_reputation;

    let mut available_roles = Vec::new();

    for (index, &threshold) in config.role_thresholds.iter().enumerate() {
        let role_level = (index + 1) as u8;
        if user_reputation.total_score >= threshold && user_reputation.role_level < role_level {
            available_roles.push(role_level);
        }
    }

    msg!(
        "User {} available role unlocks: {:?}",
        user_reputation.user,
        available_roles
    );

    Ok(available_roles)
}

#[derive(Accounts)]
#[instruction(role_level: u8)]
pub struct ClaimRoleUnlock<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        mut,
        seeds = [b"user_reputation", user.key().as_ref()],
        bump,
        constraint = user_reputation.user == user.key() @ ReputationError::UnauthorizedAdmin
    )]
    pub user_reputation: Account<'info, UserReputation>,

    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(role_level: u8)]
pub struct CheckRoleRequirements<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    /// CHECK: User account being checked
    pub user: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct GetAvailableRoleUnlocks<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    /// CHECK: User account being checked
    pub user: AccountInfo<'info>,
}