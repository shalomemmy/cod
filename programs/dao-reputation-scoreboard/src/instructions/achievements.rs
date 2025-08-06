use anchor_lang::prelude::*;
// use anchor_lang::system_program::{System};
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Award achievement badges to users
pub fn award_achievement(
    ctx: Context<AwardAchievement>,
    user: Pubkey,
    achievement_type: AchievementType,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_reputation = &mut ctx.accounts.user_reputation;

    // Validate admin authority for manual achievement awards
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Check if user already has this achievement
    require!(
        !user_reputation.has_achievement(achievement_type),
        ReputationError::AchievementAlreadyAwarded
    );

    // Award the achievement
    user_reputation.award_achievement(achievement_type);
    user_reputation.last_updated = ReputationUtils::get_current_timestamp();

    // Optionally award bonus points for achievements
    let bonus_points = match achievement_type {
        AchievementType::FirstVote => 50,
        AchievementType::WeeklyStreak => 100,
        AchievementType::MonthlyStreak => 500,
        AchievementType::TopContributor => 1000,
        AchievementType::ConsistentVoter => 200,
        AchievementType::CategoryExpert => 300,
        AchievementType::SeasonWinner => 2000,
        AchievementType::CommunityBuilder => 750,
    };

    // Add bonus points to governance category (achievements are governance-related)
    user_reputation.category_points[0] += bonus_points;
    user_reputation.calculate_total_score(&config.category_weights);

    // Update role level if changed
    user_reputation.role_level = ReputationUtils::calculate_role_level(
        user_reputation.total_score,
        &config.role_thresholds,
    );

    msg!(
        "Achievement {:?} awarded to user {} by admin {} (bonus: {} points)",
        achievement_type,
        user,
        ctx.accounts.admin.key(),
        bonus_points
    );

    Ok(())
}

/// Check which achievements a user has earned
pub fn check_user_achievements(
    ctx: Context<CheckUserAchievements>,
) -> Result<Vec<AchievementType>> {
    let user_reputation = &ctx.accounts.user_reputation;
    let mut earned_achievements = Vec::new();

    // Check all achievement types
    let all_achievements = [
        AchievementType::FirstVote,
        AchievementType::WeeklyStreak,
        AchievementType::MonthlyStreak,
        AchievementType::TopContributor,
        AchievementType::ConsistentVoter,
        AchievementType::CategoryExpert,
        AchievementType::SeasonWinner,
        AchievementType::CommunityBuilder,
    ];

    for achievement in all_achievements.iter() {
        if user_reputation.has_achievement(*achievement) {
            earned_achievements.push(*achievement);
        }
    }

    msg!(
        "User {} has earned {} achievements",
        user_reputation.user,
        earned_achievements.len()
    );

    Ok(earned_achievements)
}

/// Automatically check and award achievements based on user stats
pub fn auto_award_achievements(
    ctx: Context<AutoAwardAchievements>,
) -> Result<Vec<AchievementType>> {
    let user_reputation = &mut ctx.accounts.user_reputation;
    let mut newly_awarded = Vec::new();

    // Check for automatic achievements
    let achievements_to_check = [
        AchievementType::FirstVote,
        AchievementType::WeeklyStreak,
        AchievementType::MonthlyStreak,
        AchievementType::TopContributor,
        AchievementType::ConsistentVoter,
        AchievementType::CategoryExpert,
        AchievementType::CommunityBuilder,
    ];

    for achievement in achievements_to_check.iter() {
        if !user_reputation.has_achievement(*achievement) &&
           ReputationUtils::should_award_achievement(user_reputation, *achievement) {
            user_reputation.award_achievement(*achievement);
            newly_awarded.push(*achievement);
        }
    }

    if !newly_awarded.is_empty() {
        user_reputation.last_updated = ReputationUtils::get_current_timestamp();
        
        msg!(
            "Auto-awarded {} achievements to user {}",
            newly_awarded.len(),
            user_reputation.user
        );
    }

    Ok(newly_awarded)
}

/// Get achievement progress for a user
pub fn get_achievement_progress(
    ctx: Context<GetAchievementProgress>,
) -> Result<Vec<AchievementProgress>> {
    let user_reputation = &ctx.accounts.user_reputation;
    let mut progress_list = Vec::new();

    // FirstVote achievement
    progress_list.push(AchievementProgress {
        achievement_type: AchievementType::FirstVote,
        is_earned: user_reputation.has_achievement(AchievementType::FirstVote),
        progress_value: user_reputation.votes_cast.min(1),
        required_value: 1,
        progress_percentage: if user_reputation.votes_cast > 0 { 100 } else { 0 },
    });

    // WeeklyStreak achievement
    progress_list.push(AchievementProgress {
        achievement_type: AchievementType::WeeklyStreak,
        is_earned: user_reputation.has_achievement(AchievementType::WeeklyStreak),
        progress_value: user_reputation.current_streak.min(7) as u64,
        required_value: 7,
        progress_percentage: ((user_reputation.current_streak.min(7) as f64 / 7.0) * 100.0) as u8,
    });

    // MonthlyStreak achievement
    progress_list.push(AchievementProgress {
        achievement_type: AchievementType::MonthlyStreak,
        is_earned: user_reputation.has_achievement(AchievementType::MonthlyStreak),
        progress_value: user_reputation.current_streak.min(30) as u64,
        required_value: 30,
        progress_percentage: ((user_reputation.current_streak.min(30) as f64 / 30.0) * 100.0) as u8,
    });

    // TopContributor achievement
    progress_list.push(AchievementProgress {
        achievement_type: AchievementType::TopContributor,
        is_earned: user_reputation.has_achievement(AchievementType::TopContributor),
        progress_value: user_reputation.total_score.min(10000),
        required_value: 10000,
        progress_percentage: ((user_reputation.total_score.min(10000) as f64 / 10000.0) * 100.0) as u8,
    });

    // ConsistentVoter achievement
    progress_list.push(AchievementProgress {
        achievement_type: AchievementType::ConsistentVoter,
        is_earned: user_reputation.has_achievement(AchievementType::ConsistentVoter),
        progress_value: user_reputation.votes_cast.min(100),
        required_value: 100,
        progress_percentage: ((user_reputation.votes_cast.min(100) as f64 / 100.0) * 100.0) as u8,
    });

    // CategoryExpert achievement
    let max_category_points = *user_reputation.category_points.iter().max().unwrap_or(&0);
    progress_list.push(AchievementProgress {
        achievement_type: AchievementType::CategoryExpert,
        is_earned: user_reputation.has_achievement(AchievementType::CategoryExpert),
        progress_value: max_category_points.min(5000),
        required_value: 5000,
        progress_percentage: ((max_category_points.min(5000) as f64 / 5000.0) * 100.0) as u8,
    });

    // CommunityBuilder achievement
    let community_points = user_reputation.category_points[ReputationCategory::Community.to_index()];
    progress_list.push(AchievementProgress {
        achievement_type: AchievementType::CommunityBuilder,
        is_earned: user_reputation.has_achievement(AchievementType::CommunityBuilder),
        progress_value: community_points.min(3000),
        required_value: 3000,
        progress_percentage: ((community_points.min(3000) as f64 / 3000.0) * 100.0) as u8,
    });

    Ok(progress_list)
}

/// Remove achievement from user (admin only)
pub fn revoke_achievement(
    ctx: Context<RevokeAchievement>,
    user: Pubkey,
    achievement_type: AchievementType,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_reputation = &mut ctx.accounts.user_reputation;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Check if user has this achievement
    require!(
        user_reputation.has_achievement(achievement_type),
        ReputationError::AchievementAlreadyAwarded
    );

    // Remove the achievement
    let bit_position = achievement_type as u8;
    user_reputation.achievements &= !(1u64 << bit_position);
    user_reputation.last_updated = ReputationUtils::get_current_timestamp();

    msg!(
        "Achievement {:?} revoked from user {} by admin {}",
        achievement_type,
        user,
        ctx.accounts.admin.key()
    );

    Ok(())
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
// pub struct AchievementProgress {
//     pub achievement_type: AchievementType,
//     pub earned: bool,
//     pub current_progress: u64,
//     pub required_progress: u64,
//     pub progress_percentage: u8,
// }

#[derive(Accounts)]
#[instruction(user: Pubkey, achievement_type: AchievementType)]
pub struct AwardAchievement<'info> {
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
pub struct CheckUserAchievements<'info> {
    #[account(
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    /// CHECK: User account being checked
    pub user: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AutoAwardAchievements<'info> {
    #[account(
        mut,
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    /// CHECK: User account being processed
    pub user: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct GetAchievementProgress<'info> {
    #[account(
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    /// CHECK: User account being checked
    pub user: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(user: Pubkey, achievement_type: AchievementType)]
pub struct RevokeAchievement<'info> {
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