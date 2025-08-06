use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Apply reputation decay for inactive users
pub fn apply_reputation_decay(
    ctx: Context<ApplyReputationDecay>,
    user: Pubkey,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_reputation = &mut ctx.accounts.user_reputation;
    let current_time = ReputationUtils::get_current_timestamp();

    // Check if decay is enabled
    require!(
        config.decay_enabled,
        ReputationError::ReputationDecayDisabled
    );

    // Check if user has any activity to decay
    require!(
        user_reputation.last_activity > 0,
        ReputationError::NoActivityToDecay
    );

    // Calculate days since last activity
    let days_inactive = (current_time - user_reputation.last_activity) / 86400;
    
    // Only apply decay if user has been inactive for at least 1 day
    require!(
        days_inactive > 0,
        ReputationError::NoActivityToDecay
    );

    // Calculate decay factor
    let decay_factor = ReputationUtils::calculate_decay_factor(
        user_reputation.last_activity,
        current_time,
        config.decay_rate,
    );

    // Apply decay to all category points
    let mut total_decayed = 0u64;
    let mut new_category_points = user_reputation.category_points;
    let mut new_raw_votes = user_reputation.raw_votes;

    for (i, points) in new_category_points.iter_mut().enumerate() {
        let original_points = *points;
        *points = (original_points * decay_factor) / 10000;
        total_decayed += original_points - *points;
        
        // Update raw votes proportionally
        new_raw_votes[i] = (new_raw_votes[i] * decay_factor) / 10000;
    }

    // Update the user reputation with new values
    user_reputation.category_points = new_category_points;
    user_reputation.raw_votes = new_raw_votes;

    // Recalculate total score with new category points
    user_reputation.calculate_total_score(&config.category_weights);

    // Update role level based on new score
    user_reputation.role_level = ReputationUtils::calculate_role_level(
        user_reputation.total_score,
        &config.role_thresholds,
    );

    // Update last updated timestamp
    user_reputation.last_updated = current_time;

    msg!(
        "Reputation decay applied to user {}: {} days inactive, {} points decayed",
        user,
        days_inactive,
        total_decayed
    );

    Ok(())
}

/// Calculate potential decay without applying it
pub fn calculate_decay_preview(
    ctx: Context<CalculateDecayPreview>,
    user: Pubkey,
) -> Result<DecayPreview> {
    let config = &ctx.accounts.config;
    let user_reputation = &ctx.accounts.user_reputation;
    let current_time = ReputationUtils::get_current_timestamp();

    let days_inactive = if user_reputation.last_activity > 0 {
        (current_time - user_reputation.last_activity) / 86400
    } else {
        0
    };

    let decay_factor = if days_inactive > 0 && config.decay_enabled {
        ReputationUtils::calculate_decay_factor(
            user_reputation.last_activity,
            current_time,
            config.decay_rate,
        )
    } else {
        10000 // No decay
    };

    let mut projected_points = [0u64; 4];
    let mut total_decay = 0u64;

    for (i, &points) in user_reputation.category_points.iter().enumerate() {
        projected_points[i] = (points * decay_factor) / 10000;
        total_decay += points - projected_points[i];
    }

    // Calculate projected total score
    let mut projected_total_score = 0u64;
    for (i, &points) in projected_points.iter().enumerate() {
        let scaled_points = if points == 0 {
            0
        } else {
            ReputationUtils::calculate_quadratic_weight(points)
        };
        projected_total_score += scaled_points * (config.category_weights[i] as u64);
    }
    projected_total_score /= 10000;

    let projected_role_level = ReputationUtils::calculate_role_level(
        projected_total_score,
        &config.role_thresholds,
    );

    // Calculate decay amounts for preview
    let mut decay_amounts = [0u64; 4];
    let mut new_category_points = user_reputation.category_points;
    
    if config.decay_enabled && days_inactive >= 7 {
        let decay_factor = 9500; // 5% decay = 95% remaining
        for i in 0..4 {
            let decay_amount = (user_reputation.category_points[i] * (10000 - decay_factor)) / 10000;
            decay_amounts[i] = decay_amount;
            new_category_points[i] = user_reputation.category_points[i] - decay_amount;
        }
    }

    let preview = DecayPreview {
        current_points: user_reputation.category_points,
        points_after_decay: new_category_points,
        decay_amount: decay_amounts,
        days_since_activity: days_inactive as u64,
        will_decay: config.decay_enabled && days_inactive >= 7,
    };

    Ok(preview)
}

/// Reset decay for a user (admin function)
pub fn reset_decay_timer(
    ctx: Context<ResetDecayTimer>,
    user: Pubkey,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_reputation = &mut ctx.accounts.user_reputation;
    let current_time = ReputationUtils::get_current_timestamp();

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Reset last activity to current time
    user_reputation.last_activity = current_time;
    user_reputation.last_updated = current_time;

    msg!(
        "Decay timer reset for user {} by admin {}",
        user,
        ctx.accounts.admin.key()
    );

    Ok(())
}

/// Get decay information for multiple users
pub fn get_decay_status(
    ctx: Context<GetDecayStatus>,
    users: Vec<Pubkey>,
) -> Result<Vec<DecayStatus>> {
    let config = &ctx.accounts.config;
    let current_time = ReputationUtils::get_current_timestamp();

    // Limit the number of users to process to avoid stack overflow
    let limited_users = if users.len() > 10 {
        &users[0..10]
    } else {
        &users[..]
    };

    let mut decay_statuses = Vec::with_capacity(limited_users.len());

    // In a real implementation, you would load each user's reputation account
    // For this example, we'll return mock data
    for user in limited_users {
        let status = DecayStatus {
            user: *user,
            last_activity: current_time - (7 * 86400), // Mock: 7 days ago
            days_inactive: 7,
            decay_pending: config.decay_enabled,
            next_decay_amount: [100, 150, 200, 50], // Mock decay amounts for each category
        };
        decay_statuses.push(status);
    }

    Ok(decay_statuses)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
// pub struct DecayPreview {
//     pub user: Pubkey,
//     pub current_total_score: u64,
//     pub projected_total_score: u64,
//     pub current_role_level: u8,
//     pub projected_role_level: u8,
//     pub days_inactive: u32,
//     pub total_points_to_decay: u64,
//     pub decay_factor: u64,
//     pub decay_enabled: bool,
// }

// DecayStatus is now defined in state.rs - removed duplicate

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct ApplyReputationDecay<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        mut,
        seeds = [b"user_reputation", user.as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct CalculateDecayPreview<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        seeds = [b"user_reputation", user.as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct ResetDecayTimer<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        mut,
        seeds = [b"user_reputation", user.as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    #[account(
        constraint = admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(users: Vec<Pubkey>)]
pub struct GetDecayStatus<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,
}