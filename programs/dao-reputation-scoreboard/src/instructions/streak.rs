use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Update user streak for consecutive participation
pub fn update_user_streak(
    ctx: Context<UpdateUserStreak>,
    user: Pubkey,
) -> Result<()> {
    let user_reputation = &mut ctx.accounts.user_reputation;
    let current_time = ReputationUtils::get_current_timestamp();
    let current_day = current_time / 86400; // Convert to days
    let last_activity_day = user_reputation.last_activity / 86400;

    let mut streak_bonus = 0u64;
    let mut newly_awarded_achievements = Vec::new();

    // Calculate streak based on consecutive days of activity
    if last_activity_day == current_day {
        // Same day, no streak update needed
        return Ok(());
    } else if last_activity_day == current_day - 1 {
        // Consecutive day, increment streak
        user_reputation.current_streak += 1;
        
        // Update longest streak if current streak is better
        if user_reputation.current_streak > user_reputation.longest_streak {
            user_reputation.longest_streak = user_reputation.current_streak;
        }

        // Calculate streak bonus
        streak_bonus = ReputationUtils::calculate_streak_bonus(user_reputation.current_streak);

        // Check for streak achievements
        if user_reputation.current_streak == 7 && 
           !user_reputation.has_achievement(AchievementType::WeeklyStreak) {
            user_reputation.award_achievement(AchievementType::WeeklyStreak);
            newly_awarded_achievements.push(AchievementType::WeeklyStreak);
        }

        if user_reputation.current_streak == 30 && 
           !user_reputation.has_achievement(AchievementType::MonthlyStreak) {
            user_reputation.award_achievement(AchievementType::MonthlyStreak);
            newly_awarded_achievements.push(AchievementType::MonthlyStreak);
        }

    } else if last_activity_day < current_day - 1 {
        // Streak broken, reset to 1 (current day counts as new streak start)
        user_reputation.current_streak = 1;
    }

    // Apply streak bonus to governance category (participation bonus)
    if streak_bonus > 0 {
        user_reputation.category_points[0] += streak_bonus;
        user_reputation.raw_votes[0] += streak_bonus;
    }

    // Update activity timestamp
    user_reputation.last_activity = current_time;
    user_reputation.last_updated = current_time;

    msg!(
        "Streak updated for user {}: {} days (bonus: {} points, achievements: {})",
        user,
        user_reputation.current_streak,
        streak_bonus,
        newly_awarded_achievements.len()
    );

    Ok(())
}

/// Get user's streak information
pub fn get_user_streak_info(
    ctx: Context<GetUserStreakInfo>,
) -> Result<StreakInfo> {
    let user_reputation = &ctx.accounts.user_reputation;
    let current_time = ReputationUtils::get_current_timestamp();
    let current_day = current_time / 86400;
    let last_activity_day = user_reputation.last_activity / 86400;

    // Determine if streak is at risk
    let days_since_activity = current_day - last_activity_day;
    let streak_at_risk = days_since_activity >= 1;
    let streak_broken = days_since_activity > 1;

    // Calculate potential bonus for next day
    let next_day_bonus = if !streak_broken {
        ReputationUtils::calculate_streak_bonus(user_reputation.current_streak + 1)
    } else {
        ReputationUtils::calculate_streak_bonus(1)
    };

    let streak_info = StreakInfo {
        user: user_reputation.user,
        current_streak: user_reputation.current_streak,
        longest_streak: user_reputation.longest_streak,
        days_since_last_activity: days_since_activity as u32,
        streak_at_risk,
        streak_broken,
        current_streak_bonus: ReputationUtils::calculate_streak_bonus(user_reputation.current_streak),
        next_day_bonus,
        last_activity: user_reputation.last_activity,
    };

    Ok(streak_info)
}

/// Reset user streak (admin function)
pub fn reset_user_streak(
    ctx: Context<ResetUserStreak>,
    user: Pubkey,
    new_streak: u32,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_reputation = &mut ctx.accounts.user_reputation;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Validate new streak value
    require!(new_streak <= 365, ReputationError::InvalidConfigurationValues);

    let old_streak = user_reputation.current_streak;
    user_reputation.current_streak = new_streak;

    // Update longest streak if applicable
    if new_streak > user_reputation.longest_streak {
        user_reputation.longest_streak = new_streak;
    }

    user_reputation.last_updated = ReputationUtils::get_current_timestamp();

    msg!(
        "Streak reset for user {} by admin {}: {} -> {}",
        user,
        ctx.accounts.admin.key(),
        old_streak,
        new_streak
    );

    Ok(())
}

/// Get streak leaderboard (top streaks)
pub fn get_streak_leaderboard(
    ctx: Context<GetStreakLeaderboard>,
    leaderboard_type: StreakLeaderboardType,
) -> Result<Vec<StreakLeaderboardEntry>> {
    // In a real implementation, this would query all user reputation accounts
    // and sort by current_streak or longest_streak
    
    let mock_entries = vec![
        StreakLeaderboardEntry {
            user: Pubkey::default(),
            current_streak: 45,
            longest_streak: 67,
            streak_bonus: ReputationUtils::calculate_streak_bonus(45),
            rank: 1,
        },
        StreakLeaderboardEntry {
            user: Pubkey::default(),
            current_streak: 32,
            longest_streak: 89,
            streak_bonus: ReputationUtils::calculate_streak_bonus(32),
            rank: 2,
        },
        StreakLeaderboardEntry {
            user: Pubkey::default(),
            current_streak: 28,
            longest_streak: 34,
            streak_bonus: ReputationUtils::calculate_streak_bonus(28),
            rank: 3,
        },
    ];

    msg!(
        "Streak leaderboard requested: {:?} ({} entries)",
        leaderboard_type,
        mock_entries.len()
    );

    Ok(mock_entries)
}

/// Bulk update streaks for multiple users (admin function)
pub fn bulk_update_streaks(
    ctx: Context<BulkUpdateStreaks>,
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
        users.len() <= 100,
        ReputationError::BulkOperationTooLarge
    );

    msg!(
        "Bulk streak update initiated by admin: {} ({} users)",
        ctx.accounts.admin.key(),
        users.len()
    );

    // In a real implementation, you would iterate through each user
    // and update their streak based on their activity

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum StreakLeaderboardType {
    CurrentStreak,
    LongestStreak,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StreakInfo {
    pub user: Pubkey,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub days_since_last_activity: u32,
    pub streak_at_risk: bool,
    pub streak_broken: bool,
    pub current_streak_bonus: u64,
    pub next_day_bonus: u64,
    pub last_activity: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StreakLeaderboardEntry {
    pub user: Pubkey,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub streak_bonus: u64,
    pub rank: u32,
}

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct UpdateUserStreak<'info> {
    #[account(
        mut,
        seeds = [b"user_reputation", user.as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,
}

#[derive(Accounts)]
pub struct GetUserStreakInfo<'info> {
    #[account(
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    /// CHECK: User account being queried
    pub user: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(user: Pubkey, new_streak: u32)]
pub struct ResetUserStreak<'info> {
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
#[instruction(leaderboard_type: StreakLeaderboardType)]
pub struct GetStreakLeaderboard<'info> {
    // No specific accounts needed for leaderboard query
}

#[derive(Accounts)]
#[instruction(users: Vec<Pubkey>)]
pub struct BulkUpdateStreaks<'info> {
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