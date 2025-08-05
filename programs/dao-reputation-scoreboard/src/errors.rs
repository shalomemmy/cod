use anchor_lang::prelude::*;

/// Custom error codes for the DAO Reputation Scoreboard program
#[error_code]
pub enum ReputationError {
    #[msg("Unauthorized: Only admin can perform this action")]
    UnauthorizedAdmin,
    
    #[msg("Account is too new to participate in voting")]
    AccountTooNew,
    
    #[msg("User is still in cooldown period for voting")]
    VotingCooldownNotExpired,
    
    #[msg("Daily vote limit exceeded")]
    DailyVoteLimitExceeded,
    
    #[msg("Insufficient reputation to vote on others")]
    InsufficientReputationToVote,
    
    #[msg("Cannot vote on yourself")]
    CannotVoteOnSelf,
    
    #[msg("Invalid vote weight provided")]
    InvalidVoteWeight,
    
    #[msg("Invalid reputation category")]
    InvalidReputationCategory,
    
    #[msg("Invalid role level specified")]
    InvalidRoleLevel,
    
    #[msg("User does not meet role unlock requirements")]
    RoleUnlockRequirementsNotMet,
    
    #[msg("Season is not currently active")]
    SeasonNotActive,
    
    #[msg("Season duration exceeds maximum allowed")]
    SeasonDurationTooLong,
    
    #[msg("Invalid pagination parameters")]
    InvalidPaginationParameters,
    
    #[msg("Configuration values are out of valid range")]
    InvalidConfigurationValues,
    
    #[msg("Bulk operation size exceeds maximum allowed")]
    BulkOperationTooLarge,
    
    #[msg("Points change would result in negative reputation")]
    NegativeReputationNotAllowed,
    
    #[msg("Achievement already awarded to user")]
    AchievementAlreadyAwarded,
    
    #[msg("User reputation account not found")]
    UserReputationNotFound,
    
    #[msg("Voting record not found")]
    VotingRecordNotFound,
    
    #[msg("Season data not found")]
    SeasonDataNotFound,
    
    #[msg("Reputation system not initialized")]
    SystemNotInitialized,
    
    #[msg("Reputation system already initialized")]
    SystemAlreadyInitialized,
    
    #[msg("Invalid timestamp provided")]
    InvalidTimestamp,
    
    #[msg("String exceeds maximum allowed length")]
    StringTooLong,
    
    #[msg("Numerical overflow occurred in calculation")]
    NumericalOverflow,
    
    #[msg("Division by zero attempted")]
    DivisionByZero,
    
    #[msg("Account initialization failed")]
    AccountInitializationFailed,
    
    #[msg("Reputation decay is not enabled")]
    ReputationDecayDisabled,
    
    #[msg("User has no activity to decay")]
    NoActivityToDecay,
    
    #[msg("Export operation failed")]
    ExportOperationFailed,
    
    #[msg("Invalid signature for reputation certificate")]
    InvalidCertificateSignature,
    
    #[msg("Leaderboard calculation failed")]
    LeaderboardCalculationFailed,
    
    #[msg("Memory allocation failed")]
    MemoryAllocationFailed,
    
    #[msg("Maximum users limit reached")]
    MaxUsersLimitReached,
    
    #[msg("Category weights do not sum to expected total")]
    InvalidCategoryWeights,
    
    #[msg("Role thresholds are not in ascending order")]
    InvalidRoleThresholds,
}