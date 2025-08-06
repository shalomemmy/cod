use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

/// Utility functions for the DAO Reputation Scoreboard program
pub struct ReputationUtils;

impl ReputationUtils {
    /// Validate that category weights sum to 10000 basis points (100%)
    pub fn validate_category_weights(weights: &[u16; 4]) -> Result<()> {
        let sum: u32 = weights.iter().map(|&w| w as u32).sum();
        if sum != 10000 {
            return err!(ReputationError::InvalidCategoryWeights);
        }
        Ok(())
    }

    /// Validate that role thresholds are in ascending order
    pub fn validate_role_thresholds(thresholds: &[u64; 3]) -> Result<()> {
        for i in 1..3 {
            if thresholds[i] <= thresholds[i - 1] {
                return err!(ReputationError::InvalidRoleThresholds);
            }
        }
        Ok(())
    }

    /// Calculate user's role level based on total score and thresholds
    pub fn calculate_role_level(total_score: u64, thresholds: &[u64; 3]) -> u8 {
        for (level, &threshold) in thresholds.iter().enumerate().rev() {
            if total_score >= threshold {
                return (level + 1) as u8;
            }
        }
        0
    }

    /// Check if account meets minimum age requirement
    pub fn check_account_age(account_created: i64, min_age: u64, current_time: i64) -> Result<()> {
        let account_age = (current_time - account_created) as u64;
        if account_age < min_age {
            return err!(ReputationError::AccountTooNew);
        }
        Ok(())
    }

    /// Check if user is in voting cooldown period
    pub fn check_voting_cooldown(last_vote: i64, cooldown: u64, current_time: i64) -> Result<()> {
        let time_since_last_vote = (current_time - last_vote) as u64;
        if time_since_last_vote < cooldown {
            return err!(ReputationError::VotingCooldownNotExpired);
        }
        Ok(())
    }

    /// Calculate streak bonus based on consecutive days of activity
    pub fn calculate_streak_bonus(streak: u32) -> u64 {
        match streak {
            0..=6 => 0,      // No bonus for less than a week
            7..=13 => 100,   // 1 week streak
            14..=29 => 300,  // 2+ week streak
            30..=89 => 500,  // 1+ month streak
            90..=179 => 800, // 3+ month streak
            180.. => 1000,   // 6+ month streak
        }
    }

    /// Generate deterministic hash for reputation certificate
    pub fn generate_certificate_hash(
        user: &Pubkey,
        total_score: u64,
        category_scores: &[u64; 4],
        timestamp: i64,
        program_id: &Pubkey,
    ) -> [u8; 32] {
        use anchor_lang::solana_program::hash::hash;
        
        let mut data = Vec::new();
        data.extend_from_slice(&user.to_bytes());
        data.extend_from_slice(&total_score.to_le_bytes());
        for &score in category_scores {
            data.extend_from_slice(&score.to_le_bytes());
        }
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&program_id.to_bytes());
        
        hash(&data).to_bytes()
    }

    /// Validate string length for program inputs
    pub fn validate_string_length(s: &str, max_length: usize) -> Result<()> {
        if s.len() > max_length {
            return err!(ReputationError::StringTooLong);
        }
        Ok(())
    }

    /// Safely add points to prevent overflow
    pub fn safe_add_points(current: u64, addition: u64) -> Result<u64> {
        current.checked_add(addition)
            .ok_or(ReputationError::NumericalOverflow.into())
    }

    /// Safely subtract points to prevent underflow
    pub fn safe_subtract_points(current: u64, subtraction: u64) -> Result<u64> {
        if subtraction > current {
            return err!(ReputationError::NegativeReputationNotAllowed);
        }
        Ok(current - subtraction)
    }

    /// Check if user meets minimum reputation requirement
    pub fn check_minimum_reputation(user_score: u64, minimum: u64) -> Result<()> {
        if user_score < minimum {
            return err!(ReputationError::InsufficientReputationToVote);
        }
        Ok(())
    }

    /// Calculate quadratic voting weight to prevent whale dominance
    pub fn calculate_quadratic_weight(raw_votes: u64) -> u64 {
        if raw_votes == 0 {
            return 0;
        }
        
        // Integer square root approximation using binary search
        let mut low = 0u64;
        let mut high = raw_votes;
        
        while low <= high {
            let mid = (low + high) / 2;
            let square = mid * mid;
            
            if square == raw_votes {
                return mid * 100; // Scale for precision
            } else if square < raw_votes {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        
        high * 100 // Scale for precision
    }

    /// Get current Unix timestamp
    pub fn get_current_timestamp() -> i64 {
        Clock::get().unwrap().unix_timestamp
    }

    /// Calculate reputation decay factor based on inactivity
    pub fn calculate_decay_factor(
        last_activity: i64,
        current_time: i64,
        decay_rate: u16,
    ) -> u64 {
        let days_inactive = ((current_time - last_activity) / 86400).max(0) as u32;
        
        if days_inactive == 0 {
            return 10000; // No decay
        }
        
        // Apply compound decay: (1 - decay_rate/10000)^days
        let decay_per_day = 10000 - decay_rate as u64;
        let mut factor = 10000u64;
        
        for _ in 0..days_inactive {
            factor = (factor * decay_per_day) / 10000;
        }
        
        factor
    }

    /// Sort leaderboard entries by total score (descending)
    pub fn sort_leaderboard(entries: &mut [LeaderboardEntry]) {
        entries.sort_by(|a, b| b.score.cmp(&a.score));
        
        // Update ranks
        for (i, entry) in entries.iter_mut().enumerate() {
            entry.rank = i as u32 + 1;
        }
    }

    /// Validate vote weight is within acceptable range
    pub fn validate_vote_weight(weight: u8) -> Result<()> {
        if weight == 0 || weight > 10 {
            return err!(ReputationError::InvalidVoteWeight);
        }
        Ok(())
    }

    /// Check if achievement should be awarded based on user stats
    pub fn should_award_achievement(
        user: &UserReputation,
        achievement: AchievementType,
    ) -> bool {
        match achievement {
            AchievementType::FirstVote => user.votes_cast == 1,
            AchievementType::WeeklyStreak => user.current_streak >= 7,
            AchievementType::MonthlyStreak => user.current_streak >= 30,
            AchievementType::TopContributor => user.total_score >= 10000,
            AchievementType::ConsistentVoter => user.votes_cast >= 100,
            AchievementType::CategoryExpert => {
                user.category_points.iter().any(|&points| points >= 5000)
            },
            AchievementType::SeasonWinner => false, // Handled separately
            AchievementType::CommunityBuilder => {
                user.category_points[ReputationCategory::Community.to_index()] >= 3000
            },
        }
    }

    /// Calculate seasonal bonus based on performance
    pub fn calculate_seasonal_bonus(rank: u32, total_participants: u32) -> u64 {
        if total_participants == 0 {
            return 0;
        }
        
        let percentile = (rank as f64 / total_participants as f64) * 100.0;
        
        match percentile as u32 {
            0..=5 => 2000,   // Top 5%
            6..=10 => 1500,  // Top 10%
            11..=25 => 1000, // Top 25%
            26..=50 => 500,  // Top 50%
            _ => 100,        // Participation bonus
        }
    }

    /// Validate pagination parameters
    pub fn validate_pagination(_page: u32, page_size: u8) -> Result<()> {
        if page_size == 0 || page_size > 100 {
            return err!(ReputationError::InvalidPaginationParameters);
        }
        Ok(())
    }

    /// Calculate offset for pagination
    pub fn calculate_pagination_offset(page: u32, page_size: u8) -> Result<usize> {
        let offset = page.checked_mul(page_size as u32)
            .ok_or(ReputationError::NumericalOverflow)?;
        Ok(offset as usize)
    }
}