use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;
pub mod utils;

use instructions::*;

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
        season_id: u32,
    ) -> Result<()> {
        instructions::start_new_season(ctx, season_name, duration_days, season_id)
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

    /// End current season and distribute rewards
    pub fn end_current_season(
        ctx: Context<EndCurrentSeason>,
        season_id: u32,
    ) -> Result<()> {
        instructions::end_current_season(ctx, season_id)
    }

    /// Get current season information
    pub fn get_season_info(
        ctx: Context<GetSeasonInfo>,
        season_id: u32,
    ) -> Result<SeasonInfo> {
        instructions::get_season_info(ctx, season_id)
    }

    /// Reset user seasonal points (admin only)
    pub fn reset_seasonal_points(
        ctx: Context<ResetSeasonalPoints>,
    ) -> Result<()> {
        instructions::reset_seasonal_points(ctx)
    }

    /// Verify a reputation certificate's authenticity
    pub fn verify_reputation_certificate(
        ctx: Context<VerifyReputationCertificate>,
        certificate: ReputationCertificate,
    ) -> Result<bool> {
        instructions::verify_reputation_certificate(ctx, certificate)
    }

    /// Get leaderboard rankings update
    pub fn update_leaderboard_rankings(
        ctx: Context<UpdateLeaderboardRankings>,
    ) -> Result<()> {
        instructions::update_leaderboard_rankings(ctx)
    }

    /// Get user ranking
    pub fn get_user_ranking(
        ctx: Context<GetUserRanking>,
    ) -> Result<u32> {
        instructions::get_user_ranking(ctx)
    }

    /// Check role requirements
    pub fn check_role_requirements(
        ctx: Context<CheckRoleRequirements>,
        role_level: u8,
    ) -> Result<bool> {
        instructions::check_role_requirements(ctx, role_level)
    }

    /// Get available role unlocks
    pub fn get_available_role_unlocks(
        ctx: Context<GetAvailableRoleUnlocks>,
    ) -> Result<Vec<u8>> {
        instructions::get_available_role_unlocks(ctx)
    }

    /// Calculate decay preview
    pub fn calculate_decay_preview(
        ctx: Context<CalculateDecayPreview>,
        user: Pubkey,
    ) -> Result<DecayPreview> {
        instructions::calculate_decay_preview(ctx, user)
    }

    /// Reset decay timer (admin only)
    pub fn reset_decay_timer(
        ctx: Context<ResetDecayTimer>,
        user: Pubkey,
    ) -> Result<()> {
        instructions::reset_decay_timer(ctx, user)
    }

    /// Get decay status
    pub fn get_decay_status(
        ctx: Context<GetDecayStatus>,
        users: Vec<Pubkey>,
    ) -> Result<Vec<DecayStatus>> {
        instructions::get_decay_status(ctx, users)
    }

    /// Check user achievements
    pub fn check_user_achievements(
        ctx: Context<CheckUserAchievements>,
    ) -> Result<Vec<AchievementType>> {
        instructions::check_user_achievements(ctx)
    }

    /// Auto award achievements
    pub fn auto_award_achievements(
        ctx: Context<AutoAwardAchievements>,
    ) -> Result<Vec<AchievementType>> {
        instructions::auto_award_achievements(ctx)
    }

    /// Get achievement progress
    pub fn get_achievement_progress(
        ctx: Context<GetAchievementProgress>,
    ) -> Result<Vec<AchievementProgress>> {
        instructions::get_achievement_progress(ctx)
    }

    /// Revoke achievement (admin only)
    pub fn revoke_achievement(
        ctx: Context<RevokeAchievement>,
        user: Pubkey,
        achievement_type: AchievementType,
    ) -> Result<()> {
        instructions::revoke_achievement(ctx, user, achievement_type)
    }

    /// Get user streak info
    pub fn get_user_streak_info(
        ctx: Context<GetUserStreakInfo>,
    ) -> Result<StreakInfo> {
        instructions::get_user_streak_info(ctx)
    }

    /// Reset user streak (admin only)
    pub fn reset_user_streak(
        ctx: Context<ResetUserStreak>,
        user: Pubkey,
        new_streak: u32,
    ) -> Result<()> {
        instructions::reset_user_streak(ctx, user, new_streak)
    }

    /// Get streak leaderboard
    pub fn get_streak_leaderboard(
        ctx: Context<GetStreakLeaderboard>,
        leaderboard_type: StreakLeaderboardType,
    ) -> Result<Vec<StreakLeaderboardEntry>> {
        instructions::get_streak_leaderboard(ctx, leaderboard_type)
    }

    /// Bulk update streaks (admin only)
    pub fn bulk_update_streaks(
        ctx: Context<BulkUpdateStreaks>,
        users: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::bulk_update_streaks(ctx, users)
    }

    /// Get system configuration
    pub fn get_config(
        ctx: Context<GetConfig>,
    ) -> Result<ReputationConfigView> {
        instructions::get_config(ctx)
    }

    /// Transfer admin authority
    pub fn transfer_admin(
        ctx: Context<TransferAdmin>,
        new_admin: Pubkey,
    ) -> Result<()> {
        instructions::transfer_admin(ctx, new_admin)
    }

    /// Set system pause/unpause
    pub fn set_system_pause(
        ctx: Context<SetSystemPause>,
        paused: bool,
    ) -> Result<()> {
        instructions::set_system_pause(ctx, paused)
    }

    /// Bulk initialize users
    pub fn bulk_initialize_users(
        ctx: Context<BulkInitializeUsers>,
        users: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::bulk_initialize_users(ctx, users)
    }

    /// Bulk apply decay
    pub fn bulk_apply_decay(
        ctx: Context<BulkApplyDecay>,
        users: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::bulk_apply_decay(ctx, users)
    }

    /// Bulk award achievements
    pub fn bulk_award_achievements(
        ctx: Context<BulkAwardAchievements>,
        awards: Vec<AchievementAward>,
    ) -> Result<()> {
        instructions::bulk_award_achievements(ctx, awards)
    }

    /// Bulk reset seasonal data
    pub fn bulk_reset_seasonal_data(
        ctx: Context<BulkResetSeasonalData>,
        users: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::bulk_reset_seasonal_data(ctx, users)
    }

    /// Export leaderboard
    pub fn export_leaderboard(
        ctx: Context<ExportLeaderboard>,
        category: Option<ReputationCategory>,
        season_id: Option<u32>,
    ) -> Result<Vec<LeaderboardEntry>> {
        instructions::export_leaderboard(ctx, category, season_id)
    }

    /// Import reputation data
    pub fn import_reputation_data(
        ctx: Context<ImportReputationData>,
        import_data: Vec<BulkReputationUpdate>,
    ) -> Result<()> {
        instructions::import_reputation_data(ctx, import_data)
    }
}