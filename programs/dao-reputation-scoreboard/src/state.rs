use anchor_lang::prelude::*;

/// Global reputation system configuration
#[account]
pub struct ReputationConfig {
    /// Admin wallet that can modify system parameters
    pub admin: Pubkey,
    /// Minimum time between votes (in seconds)
    pub voting_cooldown: u64,
    /// Minimum account age to participate (in seconds)
    pub min_account_age: u64,
    /// Maximum votes per user per day
    pub daily_vote_limit: u8,
    /// Minimum reputation needed to vote on others
    pub min_reputation_to_vote: u64,
    /// Category weights for scoring [governance, development, community, treasury]
    pub category_weights: [u16; 4],
    /// Role unlock thresholds [member, contributor, senior, expert, leader]
    pub role_thresholds: [u64; 5],
    /// Current active season ID
    pub current_season: u32,
    /// Current season start timestamp
    pub season_start: i64,
    /// Season duration in seconds
    pub season_duration: u64,
    /// Total registered users
    pub total_users: u64,
    /// Reputation decay rate (basis points per week)
    pub decay_rate: u16,
    /// Whether decay is enabled
    pub decay_enabled: bool,
    /// Config initialization timestamp
    pub initialized_at: i64,
    /// Last config update timestamp
    pub last_updated: i64,
    /// Reserved space for future upgrades (OPTIMIZED)
    pub reserved: [u8; 4],
}

impl ReputationConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        8 + // voting_cooldown
        8 + // min_account_age
        1 + // daily_vote_limit
        8 + // min_reputation_to_vote
        (2 * 4) + // category_weights
        (8 * 5) + // role_thresholds
        4 + // current_season
        8 + // season_start
        8 + // season_duration
        8 + // total_users
        2 + // decay_rate
        1 + // decay_enabled
        8 + // initialized_at
        8 + // last_updated
        4; // reserved (OPTIMIZED)
}

/// Individual user reputation data
#[account]
pub struct UserReputation {
    /// User's wallet public key
    pub user: Pubkey,
    /// Points in each category [governance, development, community, treasury]
    pub category_points: [u64; 4],
    /// Raw vote counts received [governance, development, community, treasury]
    pub raw_votes: [u64; 4],
    /// Total calculated score
    pub total_score: u64,
    /// Current role level (0-4)
    pub role_level: u8,
    /// Number of achievements earned
    pub achievements: u32,
    /// Account creation timestamp
    pub created_at: i64,
    /// Last activity timestamp
    pub last_activity: i64,
    /// Current streak (consecutive days)
    pub current_streak: u32,
    /// Longest streak achieved
    pub longest_streak: u32,
    /// Best rank in any season
    pub best_season_rank: u32,
    /// Total votes cast by this user
    pub votes_cast: u64,
    /// Reserved space for future upgrades (OPTIMIZED)
    pub reserved: [u8; 4],
}

impl UserReputation {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        (8 * 4) + // category_points
        (8 * 4) + // raw_votes
        8 + // total_score
        1 + // role_level
        4 + // achievements
        8 + // created_at
        8 + // last_activity
        4 + // current_streak
        4 + // longest_streak
        4 + // best_season_rank
        8 + // votes_cast
        4; // reserved (OPTIMIZED)

    /// Calculate total score with category weights
    pub fn calculate_total_score(&mut self, category_weights: &[u16; 4]) -> u64 {
        let mut total = 0u64;
        for i in 0..4 {
            // Apply quadratic scaling: sqrt(raw_votes) * weight
            let scaled_votes = (self.raw_votes[i] as f64).sqrt() as u64;
            total += scaled_votes * category_weights[i] as u64;
        }
        
        // Add streak bonus
        total += self.current_streak as u64 * 10;
        
        self.total_score = total;
        total
    }

    /// Check if user has specific achievement
    pub fn has_achievement(&self, achievement: AchievementType) -> bool {
        let bit_position = achievement as u32;
        (self.achievements >> bit_position) & 1 == 1
    }

    /// Award achievement to user
    pub fn award_achievement(&mut self, achievement: AchievementType) {
        let bit_position = achievement as u32;
        self.achievements |= 1 << bit_position;
    }
}

/// Reputation categories
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ReputationCategory {
    #[default]
    Governance = 0,
    Development = 1,
    Community = 2,
    Treasury = 3,
}

/// Achievement types 
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AchievementType {
    #[default]
    FirstVote = 0,
    WeeklyStreak = 1,
    MonthlyStreak = 2,
    TopContributor = 3,
    ConsistentVoter = 4,
    CategoryExpert = 5,
    CommunityBuilder = 6,
}

/// Vote history entry
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, Debug)]
pub struct VoteHistoryEntry {
    pub category: ReputationCategory,
    pub is_upvote: bool,
    pub timestamp: i64,
}

impl VoteHistoryEntry {
    pub const LEN: usize = 1 + // category
        1 + // is_upvote  
        8; // timestamp

    pub fn new(category: ReputationCategory, is_upvote: bool, timestamp: i64) -> Self {
        Self {
            category,
            is_upvote,
            timestamp,
        }
    }
}

/// Voting record between two users
#[account]
pub struct VotingRecord {
    /// Voter's public key
    pub voter: Pubkey,
    /// Target user's public key
    pub target: Pubkey,
    /// Last vote timestamp
    pub last_vote: i64,
    /// Daily votes count
    pub daily_votes: u8,
    /// Last daily reset timestamp
    pub last_daily_reset: i64,
    /// Total votes cast on this target
    pub total_votes_on_target: u32,
    /// Vote history - OPTIMIZED to 1 entry
    pub vote_history: [VoteHistoryEntry; 1],
    /// Current history index (circular buffer)
    pub history_index: u8,
    /// Reserved space for future upgrades (OPTIMIZED)
    pub reserved: [u8; 4],
}

impl VotingRecord {
    pub const LEN: usize = 8 + // discriminator
        32 + // voter
        32 + // target
        8 + // last_vote
        1 + // daily_votes
        8 + // last_daily_reset
        4 + // total_votes_on_target
        (VoteHistoryEntry::LEN * 1) + // vote_history (OPTIMIZED)
        1 + // history_index
        4; // reserved (OPTIMIZED)

    /// Check if daily vote limit is reached
    pub fn is_daily_limit_reached(&mut self, limit: u8, current_time: i64) -> bool {
        self.update_daily_reset(current_time);
        self.daily_votes >= limit
    }

    /// Update daily vote count and reset if new day
    pub fn update_daily_reset(&mut self, current_time: i64) {
        let current_day = current_time / 86400; // seconds to days
        let last_reset_day = self.last_daily_reset / 86400;
        
        if current_day > last_reset_day {
            self.daily_votes = 0;
            self.last_daily_reset = current_time;
        }
    }

    /// Add vote to history (OPTIMIZED for 1 entry)
    pub fn add_vote_to_history(&mut self, category: ReputationCategory, is_upvote: bool, timestamp: i64) {
        let entry = VoteHistoryEntry::new(category, is_upvote, timestamp);
        self.vote_history[0] = entry; // Always use index 0 for single entry
        self.history_index = 0;
    }
}

/// Leaderboard entry
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct LeaderboardEntry {
    pub user: Pubkey,
    pub score: u64,
    pub rank: u32,
    pub category: ReputationCategory,
}

impl LeaderboardEntry {
    pub const LEN: usize = 32 + // user
        8 + // score
        4 + // rank
        1; // category
}

/// Season competition data
#[account]
pub struct SeasonData {
    /// Season identifier
    pub season_id: u32,
    /// Season name - MAXIMUM OPTIMIZED to 4 bytes
    pub season_name: [u8; 4],
    /// Season start timestamp
    pub start_time: i64,
    /// Season end timestamp
    pub end_time: i64,
    /// Whether season is currently active
    pub is_active: bool,
    /// Top performers - OPTIMIZED to 1 entry
    pub leaderboard: [LeaderboardEntry; 1],
    /// Total participants this season
    pub total_participants: u32,
    /// Whether rewards have been distributed
    pub rewards_distributed: bool,
    /// Total votes cast this season
    pub total_votes_cast: u64,
    /// Most active category this season
    pub most_active_category: ReputationCategory,
    /// Reserved space for future upgrades (OPTIMIZED)
    pub reserved: [u8; 4],
}

impl SeasonData {
    pub const LEN: usize = 8 + // discriminator
        4 + // season_id
        4 + // season_name (MAXIMUM OPTIMIZED)
        8 + // start_time
        8 + // end_time
        1 + // is_active
        (LeaderboardEntry::LEN * 1) + // leaderboard (OPTIMIZED)
        4 + // total_participants
        1 + // rewards_distributed
        8 + // total_votes_cast
        1 + // most_active_category
        4; // reserved (OPTIMIZED)
}

/// Additional types for complex operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct ReputationCertificate {
    pub user: Pubkey,
    pub total_score: u64,
    pub category_scores: [u64; 4],
    pub role_level: u8,
    pub achievements: u32,
    pub issued_at: i64,
    pub season_id: u32,
    pub signature_hash: [u8; 32],
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct ReputationConfigUpdate {
    pub voting_cooldown: Option<u64>,
    pub min_account_age: Option<u64>,
    pub daily_vote_limit: Option<u8>,
    pub min_reputation_to_vote: Option<u64>,
    pub category_weights: Option<[u16; 4]>,
    pub role_thresholds: Option<[u64; 5]>,
    pub decay_rate: Option<u16>,
    pub decay_enabled: Option<bool>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct BulkReputationUpdate {
    pub user: Pubkey,
    pub category_points: [u64; 4],
    pub achievements: u32,
    pub role_level: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct DecayPreview {
    pub current_points: [u64; 4],
    pub points_after_decay: [u64; 4],
    pub decay_amount: [u64; 4],
    pub days_since_activity: u64,
    pub will_decay: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct DecayStatus {
    pub last_activity: i64,
    pub days_inactive: u32,
    pub decay_pending: bool,
    pub next_decay_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct AchievementProgress {
    pub achievement_type: AchievementType,
    pub is_earned: bool,
    pub progress_value: u64,
    pub required_value: u64,
    pub progress_percentage: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
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
    pub streak_bonus: u64,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, Default)]
pub enum StreakLeaderboardType {
    #[default]
    Current,
    Longest,
    Active,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct StreakLeaderboardEntry {
    pub user: Pubkey,
    pub streak_value: u32,
    pub rank: u32,
    pub is_active: bool,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct AchievementAward {
    pub user: Pubkey,
    pub achievement: AchievementType,
    pub awarded_at: i64,
    pub season_id: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct SeasonInfo {
    pub season_id: u32,
    pub name: String,
    pub start_time: i64,
    pub end_time: i64,
    pub is_active: bool,
    pub total_participants: u32,
    pub total_votes: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct ReputationConfigView {
    pub admin: Pubkey,
    pub voting_cooldown: u64,
    pub min_account_age: u64,
    pub daily_vote_limit: u8,
    pub min_reputation_to_vote: u64,
    pub category_weights: [u16; 4],
    pub role_thresholds: [u64; 5],
    pub decay_enabled: bool,
    pub current_season: u32,
    pub total_users: u64,
}