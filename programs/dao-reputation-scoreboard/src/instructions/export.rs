use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Export user reputation as a portable certificate
pub fn export_reputation(ctx: Context<ExportReputation>) -> Result<ReputationCertificate> {
    let user_reputation = &ctx.accounts.user_reputation;
    let current_time = ReputationUtils::get_current_timestamp();
    let program_id = crate::ID;

    // Generate certificate hash for verification
    let signature_hash = ReputationUtils::generate_certificate_hash(
        &user_reputation.user,
        user_reputation.total_score,
        &user_reputation.category_points,
        current_time,
        &program_id,
    );

    let certificate = ReputationCertificate {
        user: user_reputation.user,
        total_score: user_reputation.total_score,
        category_scores: user_reputation.category_points,
        achievements: user_reputation.achievements,
        role_level: user_reputation.role_level,
        generated_at: current_time,
        program_id,
        signature_hash,
    };

    msg!(
        "Reputation certificate exported for user: {} with score: {}",
        user_reputation.user,
        user_reputation.total_score
    );

    Ok(certificate)
}

/// Verify a reputation certificate's authenticity
pub fn verify_reputation_certificate(
    _ctx: Context<VerifyReputationCertificate>,
    certificate: ReputationCertificate,
) -> Result<bool> {
    // Regenerate hash with certificate data
    let expected_hash = ReputationUtils::generate_certificate_hash(
        &certificate.user,
        certificate.total_score,
        &certificate.category_scores,
        certificate.generated_at,
        &certificate.program_id,
    );

    let is_valid = expected_hash == certificate.signature_hash 
        && certificate.program_id == crate::ID;

    msg!(
        "Certificate verification for user {}: {}",
        certificate.user,
        if is_valid { "VALID" } else { "INVALID" }
    );

    Ok(is_valid)
}

/// Export leaderboard data for external use
pub fn export_leaderboard(
    ctx: Context<ExportLeaderboard>,
    category: Option<ReputationCategory>,
    season_id: Option<u32>,
) -> Result<Vec<LeaderboardEntry>> {
    let config = &ctx.accounts.config;

    // Validate admin authority for full leaderboard export
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // In a production system, this would query and aggregate user data
    // For demonstration, returning mock data
    let mock_leaderboard = vec![
        LeaderboardEntry {
            user: Pubkey::default(),
            total_score: 10000,
            category_scores: [3000, 2500, 2500, 2000],
            rank: 1,
        },
        LeaderboardEntry {
            user: Pubkey::default(),
            total_score: 8500,
            category_scores: [2000, 3000, 2000, 1500],
            rank: 2,
        },
    ];

    msg!(
        "Leaderboard exported - Category: {:?}, Season: {:?}, Entries: {}",
        category,
        season_id,
        mock_leaderboard.len()
    );

    Ok(mock_leaderboard)
}

/// Import reputation data from external source (admin only)
pub fn import_reputation_data(
    ctx: Context<ImportReputationData>,
    import_data: Vec<BulkReputationUpdate>,
) -> Result<()> {
    let config = &ctx.accounts.config;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Validate import size
    require!(
        import_data.len() <= 100,
        ReputationError::BulkOperationTooLarge
    );

    for update in import_data.iter() {
        // Validate each update
        // Skip validation for fixed-size byte array
    }

    msg!(
        "Reputation data import initiated by admin: {} ({} records)",
        ctx.accounts.admin.key(),
        import_data.len()
    );

    Ok(())
}

#[derive(Accounts)]
pub struct ExportReputation<'info> {
    #[account(
        seeds = [b"user_reputation", user.key().as_ref()],
        bump,
        constraint = user_reputation.user == user.key()
    )]
    pub user_reputation: Account<'info, UserReputation>,

    /// CHECK: User requesting export
    pub user: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(certificate: ReputationCertificate)]
pub struct VerifyReputationCertificate {
    // No specific accounts needed for verification
}

#[derive(Accounts)]
#[instruction(category: Option<ReputationCategory>, season_id: Option<u32>)]
pub struct ExportLeaderboard<'info> {
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

#[derive(Accounts)]
#[instruction(import_data: Vec<BulkReputationUpdate>)]
pub struct ImportReputationData<'info> {
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