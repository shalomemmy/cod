use anchor_lang::prelude::*;

/// Reputation categories for multi-dimensional scoring
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ReputationCategory {
    #[default]
    Governance,
    Development,
    Community,
    Treasury,
}

impl ReputationCategory {
    pub fn to_index(&self) -> usize {
        match self {
            ReputationCategory::Governance => 0,
            ReputationCategory::Development => 1,
            ReputationCategory::Community => 2,
            ReputationCategory::Treasury => 3,
        }
    }
}

/// Achievement types for gamification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AchievementType {
    #[default]
    FirstVote,
    WeeklyStreak,
    MonthlyStreak,
    TopContributor,
    ConsistentVoter,
    CategoryExpert,
    SeasonWinner,
    CommunityBuilder,
}

/// Global configuration for the reputation system
#[account]
pub struct ReputationConfig {
    /// Admin authority for the program
    pub admin: Pubkey,
    /// Cooldown period between votes from the same wallet (in seconds)
    pub voting_cooldown: u64,
    /// Minimum account age before participation (in seconds)
    pub min_account_age: u64,
    /// Maximum votes per day per wallet
    pub daily_vote_limit: u8,
    /// Minimum reputation required to vote on others
    pub min_reputation_to_vote: u64,
    /// Weight multipliers for each category [Governance, Development, Community, Treasury]
    pub category_weights: [u16; 4],
    /// Role unlock thresholds
    pub role_thresholds: [u64; 5],
    /// Current season information
    pub current_season: u32,
    /// Season start timestamp
    pub season_start: i64,
    /// Season duration in seconds
    pub season_duration: u64,
    /// Total users in the system
    pub total_users: u64,
    /// Reputation decay rate (basis points per day)
    pub decay_rate: u16,
    /// Enable/disable reputation decay
    pub decay_enabled: bool,
    /// Program initialization timestamp
    pub initialized_at: i64,
    /// Last config update timestamp
    pub last_updated: i64,
    /// Reserved space for future upgrades
    pub reserved: [u8; 128],
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
        128; // reserved
}

/// Individual user reputation data
#[account]
pub struct UserReputation {
    /// User's wallet address
    pub user: Pubkey,
    /// Reputation points by category [Governance, Development, Community, Treasury]
    pub category_points: [u64; 4],
    /// Total reputation score (calculated with weights)
    pub total_score: u64,
    /// Raw vote counts received (before quadratic scaling)
    pub raw_votes: [u64; 4],
    /// User's role level based on reputation
    pub role_level: u8,
    /// Achievements earned by the user (bitfield)
    pub achievements: u64,
    /// Current streak of consecutive participation days
    pub current_streak: u32,
    /// Longest streak achieved
    pub longest_streak: u32,
    /// Last activity timestamp
    pub last_activity: i64,
    /// Account creation timestamp
    pub created_at: i64,
    /// Last reputation update timestamp
    pub last_updated: i64,
    /// Seasonal points for current season
    pub seasonal_points: [u64; 4],
    /// Previous season best rank
    pub best_season_rank: u32,
    /// Total votes cast by this user
    pub votes_cast: u64,
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

impl UserReputation {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        (8 * 4) + // category_points
        8 + // total_score
        (8 * 4) + // raw_votes
        1 + // role_level
        8 + // achievements
        4 + // current_streak
        4 + // longest_streak
        8 + // last_activity
        8 + // created_at
        8 + // last_updated
        (8 * 4) + // seasonal_points
        4 + // best_season_rank
        8 + // votes_cast
        64; // reserved

    /// Calculate total score with category weights
    pub fn calculate_total_score(&mut self, weights: &[u16; 4]) {
        let mut total = 0u64;
        for (i, &points) in self.category_points.iter().enumerate() {
            // Apply quadratic scaling to prevent whale dominance
            let scaled_points = self.quadratic_scale(points);
            total += scaled_points * (weights[i] as u64);
        }
        self.total_score = total / 10000; // Normalize weights (basis points)
    }

    /// Apply quadratic scaling: sqrt(points) * multiplier
    fn quadratic_scale(&self, points: u64) -> u64 {
        if points == 0 {
            return 0;
        }
        // Use integer square root approximation
        let sqrt_points = self.integer_sqrt(points);
        sqrt_points * 100 // Multiplier to maintain precision
    }

    /// Integer square root using binary search
    fn integer_sqrt(&self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        let mut x = n;
        let mut y = (n + 1) / 2;
        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }
        x
    }

    /// Check if user has specific achievement
    pub fn has_achievement(&self, achievement: AchievementType) -> bool {
        let bit_position = achievement as u8;
        (self.achievements & (1u64 << bit_position)) != 0
    }

    /// Award achievement to user
    pub fn award_achievement(&mut self, achievement: AchievementType) {
        let bit_position = achievement as u8;
        self.achievements |= 1u64 << bit_position;
    }

    /// Apply reputation decay based on inactivity
    pub fn apply_decay(&mut self, decay_rate: u16, current_time: i64) {
        let days_inactive = (current_time - self.last_activity) / 86400; // seconds to days
        if days_inactive > 0 {
            let decay_factor = (10000 - decay_rate as u64).pow(days_inactive as u32) / 10000u64.pow(days_inactive as u32);
            for points in self.category_points.iter_mut() {
                *points = (*points * decay_factor) / 10000;
            }
        }
    }
}

/// Voting record to track cooldowns and prevent abuse
#[account]
pub struct VotingRecord {
    /// Voter's wallet address
    pub voter: Pubkey,
    /// Target user being voted on
    pub target: Pubkey,
    /// Last vote timestamp
    pub last_vote: i64,
    /// Daily vote count (resets at midnight UTC)
    pub daily_votes: u8,
    /// Last daily reset timestamp
    pub last_daily_reset: i64,
    /// Total votes cast on this target
    pub total_votes_on_target: u32,
    /// Vote history (last 10 votes with timestamps)
    pub vote_history: [VoteHistoryEntry; 10],
    /// Current history index (circular buffer)
    pub history_index: u8,
    /// Reserved space for future upgrades
    pub reserved: [u8; 32],
}

impl VotingRecord {
    pub const LEN: usize = 8 + // discriminator
        32 + // voter
        32 + // target
        8 + // last_vote
        1 + // daily_votes
        8 + // last_daily_reset
        4 + // total_votes_on_target
        (VoteHistoryEntry::LEN * 10) + // vote_history
        1 + // history_index
        32; // reserved

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

    /// Add vote to history
    pub fn add_vote_to_history(&mut self, category: ReputationCategory, is_upvote: bool, timestamp: i64) {
        let entry = VoteHistoryEntry::new(category, is_upvote, timestamp);
        
        self.vote_history[self.history_index as usize] = entry;
        self.history_index = (self.history_index + 1) % 10;
    }
}

/// Individual vote history entry
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct VoteHistoryEntry {
    pub category: ReputationCategory,
    pub is_upvote: bool,
    pub timestamp: i64,
}

impl VoteHistoryEntry {
    pub fn new(category: ReputationCategory, is_upvote: bool, timestamp: i64) -> Self {
        Self {
            category,
            is_upvote,
            timestamp,
        }
    }
}

impl VoteHistoryEntry {
    pub const LEN: usize = 1 + // category (enum discriminant)
        1 + // is_upvote
        8; // timestamp
}

/// Seasonal competition data
#[account]
pub struct SeasonData {
    /// Season identifier
    pub season_id: u32,
    /// Season name/title
    pub season_name: String,
    /// Season start timestamp
    pub start_time: i64,
    /// Season end timestamp
    pub end_time: i64,
    /// Season status
    pub is_active: bool,
    /// Top performers in this season (top 10)
    pub leaderboard: [LeaderboardEntry; 10],
    /// Total participants this season
    pub total_participants: u32,
    /// Season rewards pool (if any)
    pub rewards_distributed: bool,
    /// Season statistics
    pub total_votes_cast: u64,
    /// Most active category this season
    pub most_active_category: ReputationCategory,
    /// Reserved space for future upgrades
    pub reserved: [u8; 64],
}

impl SeasonData {
    pub const LEN: usize = 8 + // discriminator
        4 + // season_id
        32 + // season_name (max length)
        8 + // start_time
        8 + // end_time
        1 + // is_active
        (LeaderboardEntry::LEN * 10) + // leaderboard
        4 + // total_participants
        1 + // rewards_distributed
        8 + // total_votes_cast
        1 + // most_active_category
        64; // reserved
}

/// Leaderboard entry structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct LeaderboardEntry {
    pub user: Pubkey,
    pub total_score: u64,
    pub category_scores: [u64; 4],
    pub rank: u32,
}

impl LeaderboardEntry {
    pub const LEN: usize = 32 + // user
        8 + // total_score
        (8 * 4) + // category_scores
        4; // rank
}

/// Reputation certificate for portability
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ReputationCertificate {
    pub user: Pubkey,
    pub total_score: u64,
    pub category_scores: [u64; 4],
    pub achievements: u64,
    pub role_level: u8,
    pub generated_at: i64,
    pub program_id: Pubkey,
    pub signature_hash: [u8; 32],
}

/// Configuration update structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
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

/// Bulk reputation update structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BulkReputationUpdate {
    pub user: Pubkey,
    pub category: ReputationCategory,
    pub points_change: i64,
    pub reason: String,
}

