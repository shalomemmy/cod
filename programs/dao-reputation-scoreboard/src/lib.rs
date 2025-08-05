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

    /// Cast a vote (upvote/downvote) on another user's reputation
    pub fn cast_vote(
        ctx: Context<CastVote>,
        is_upvote: bool,
        category: ReputationCategory,
        vote_weight: u8,
    ) -> Result<()> {
        instructions::cast_vote(ctx, is_upvote, category, vote_weight)
    }

    /// Admin function to manually update user reputation
    pub fn update_user_reputation(
        ctx: Context<UpdateUserReputation>,
        category: ReputationCategory,
        points_change: i64,
        reason: String,
    ) -> Result<()> {
        instructions::update_user_reputation(ctx, category, points_change, reason)
    }

    /// Initialize a new user reputation account
    pub fn initialize_user_reputation(
        ctx: Context<InitializeUserReputation>,
    ) -> Result<()> {
        instructions::initialize_user_reputation(ctx)
    }

    /// Get paginated leaderboard data
    pub fn get_leaderboard(
        ctx: Context<GetLeaderboard>,
        category: Option<ReputationCategory>,
        page: u32,
        page_size: u8,
    ) -> Result<Vec<LeaderboardEntry>> {
        instructions::get_leaderboard(ctx, category, page, page_size)
    }

    /// Allow users to claim role unlocks based on reputation thresholds
    pub fn claim_role_unlock(
        ctx: Context<ClaimRoleUnlock>,
        role_level: u8,
    ) -> Result<()> {
        instructions::claim_role_unlock(ctx, role_level)
    }

    /// Admin function to start a new seasonal competition
    pub fn start_new_season(
        ctx: Context<StartNewSeason>,
        season_name: String,
        duration_days: u32,
    ) -> Result<()> {
        instructions::start_new_season(ctx, season_name, duration_days)
    }

    /// Export user reputation as a portable certificate
    pub fn export_reputation(
        ctx: Context<ExportReputation>,
    ) -> Result<ReputationCertificate> {
        instructions::export_reputation(ctx)
    }

    /// Admin function to update system configuration
    pub fn update_config(
        ctx: Context<UpdateConfig>,
        new_config: ReputationConfigUpdate,
    ) -> Result<()> {
        instructions::update_config(ctx, new_config)
    }

    /// Bulk admin operation to update multiple users
    pub fn bulk_update_reputation(
        ctx: Context<BulkUpdateReputation>,
        updates: Vec<BulkReputationUpdate>,
    ) -> Result<()> {
        instructions::bulk_update_reputation(ctx, updates)
    }

    /// Apply reputation decay for inactive users
    pub fn apply_reputation_decay(
        ctx: Context<ApplyReputationDecay>,
        user: Pubkey,
    ) -> Result<()> {
        instructions::apply_reputation_decay(ctx, user)
    }

    /// Award achievement badges to users
    pub fn award_achievement(
        ctx: Context<AwardAchievement>,
        user: Pubkey,
        achievement_type: AchievementType,
    ) -> Result<()> {
        instructions::award_achievement(ctx, user, achievement_type)
    }

    /// Update user streak for consecutive participation
    pub fn update_user_streak(
        ctx: Context<UpdateUserStreak>,
        user: Pubkey,
    ) -> Result<()> {
        instructions::update_user_streak(ctx, user)
    }
}