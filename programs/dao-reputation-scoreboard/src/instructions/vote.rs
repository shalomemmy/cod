use anchor_lang::prelude::*;
use anchor_lang::system_program::{System};
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Cast a vote (upvote/downvote) on another user's reputation
pub fn cast_vote(
    ctx: Context<CastVote>,
    is_upvote: bool,
    category: ReputationCategory,
    vote_weight: u8,
) -> Result<()> {
    let config = &ctx.accounts.config;
    let voter_reputation = &mut ctx.accounts.voter_reputation;
    let target_reputation = &mut ctx.accounts.target_reputation;
    let voting_record = &mut ctx.accounts.voting_record;
    let current_time = ReputationUtils::get_current_timestamp();

    // Validation checks
    require!(ctx.accounts.voter.key() != ctx.accounts.target.key(), ReputationError::CannotVoteOnSelf);
    ReputationUtils::validate_vote_weight(vote_weight)?;
    
    // Check voter account age
    ReputationUtils::check_account_age(
        voter_reputation.created_at,
        config.min_account_age,
        current_time,
    )?;

    // Check voting cooldown
    ReputationUtils::check_voting_cooldown(
        voting_record.last_vote,
        config.voting_cooldown,
        current_time,
    )?;

    // Check daily vote limit
    require!(
        !voting_record.is_daily_limit_reached(config.daily_vote_limit, current_time),
        ReputationError::DailyVoteLimitExceeded
    );

    // Check minimum reputation to vote
    ReputationUtils::check_minimum_reputation(
        voter_reputation.total_score,
        config.min_reputation_to_vote,
    )?;

    // Calculate points to add/subtract based on vote weight and type
    let base_points = vote_weight as u64 * 10; // Base points per vote weight
    let streak_bonus = ReputationUtils::calculate_streak_bonus(voter_reputation.current_streak);
    let total_points = base_points + (streak_bonus / 10); // Small streak bonus

    let category_index = category.to_index();

    // Apply vote to target user
    if is_upvote {
        target_reputation.raw_votes[category_index] = ReputationUtils::safe_add_points(
            target_reputation.raw_votes[category_index],
            total_points,
        )?;
        target_reputation.category_points[category_index] = ReputationUtils::safe_add_points(
            target_reputation.category_points[category_index],
            total_points,
        )?;
    } else {
        // Downvotes have less impact to prevent abuse
        let downvote_points = total_points / 2;
        target_reputation.category_points[category_index] = ReputationUtils::safe_subtract_points(
            target_reputation.category_points[category_index],
            downvote_points,
        )?;
    }

    // Recalculate total score with weights
    target_reputation.calculate_total_score(&config.category_weights);
    
    // Update role level if changed
    let new_role_level = ReputationUtils::calculate_role_level(
        target_reputation.total_score,
        &config.role_thresholds,
    );
    target_reputation.role_level = new_role_level;

    // Update target user activity
    target_reputation.last_activity = current_time;
    target_reputation.last_updated = current_time;

    // Update seasonal points
    target_reputation.seasonal_points[category_index] = ReputationUtils::safe_add_points(
        target_reputation.seasonal_points[category_index],
        if is_upvote { total_points } else { 0 },
    )?;

    // Initialize voting record if needed
    if voting_record.voter == Pubkey::default() {
        voting_record.voter = ctx.accounts.voter.key();
        voting_record.target = ctx.accounts.target.key();
        voting_record.last_vote = 0;
        voting_record.daily_votes = 0;
        voting_record.last_daily_reset = current_time;
        voting_record.total_votes_on_target = 0;
        voting_record.vote_history = [VoteHistoryEntry::default(); 1];
        voting_record.history_index = 0;
        voting_record.reserved = [0; 2];
    }

    // Update voting record
    voting_record.last_vote = current_time;
    voting_record.daily_votes += 1;
    voting_record.total_votes_on_target += 1;
    voting_record.add_vote_to_history(category, is_upvote, current_time);

    // Update voter stats
    voter_reputation.votes_cast += 1;
    voter_reputation.last_activity = current_time;

    // Check for achievements
    if ReputationUtils::should_award_achievement(voter_reputation, AchievementType::FirstVote) {
        voter_reputation.award_achievement(AchievementType::FirstVote);
    }
    if ReputationUtils::should_award_achievement(voter_reputation, AchievementType::ConsistentVoter) {
        voter_reputation.award_achievement(AchievementType::ConsistentVoter);
    }

    msg!(
        "Vote cast: {} {} on {} in category {:?} with weight {}",
        ctx.accounts.voter.key(),
        if is_upvote { "upvoted" } else { "downvoted" },
        ctx.accounts.target.key(),
        category,
        vote_weight
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(is_upvote: bool, category: ReputationCategory, vote_weight: u8)]
pub struct CastVote<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        mut,
        seeds = [b"user_reputation", voter.key().as_ref()],
        bump
    )]
    pub voter_reputation: Account<'info, UserReputation>,

    #[account(
        mut,
        seeds = [b"user_reputation", target.key().as_ref()],
        bump
    )]
    pub target_reputation: Account<'info, UserReputation>,

    #[account(
        init_if_needed,
        payer = voter,
        space = VotingRecord::LEN,
        seeds = [b"voting_record", voter.key().as_ref(), target.key().as_ref()],
        bump
    )]
    pub voting_record: Account<'info, VotingRecord>,

    #[account(mut)]
    pub voter: Signer<'info>,
    
    /// CHECK: Target user account, validated through reputation account
    pub target: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}