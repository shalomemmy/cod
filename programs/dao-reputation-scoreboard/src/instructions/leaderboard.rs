use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Get paginated leaderboard data
pub fn get_leaderboard(
    ctx: Context<GetLeaderboard>,
    _category: Option<ReputationCategory>,
    page: u32,
    page_size: u8,
) -> Result<Vec<LeaderboardEntry>> {
    let _config = &ctx.accounts.config;
    
    // Validate pagination parameters
    ReputationUtils::validate_pagination(page, page_size)?;

    // This is a simplified implementation - in a real scenario, you'd want to
    // maintain sorted indexes or use external indexing for efficiency
    let mut leaderboard_entries = Vec::new();

    // For demonstration, we'll return mock data
    // In production, you'd iterate through all user reputation accounts
    let mock_entries = vec![
        LeaderboardEntry {
            user: Pubkey::default(),
            score: 5000,
            category: ReputationCategory::Governance,
            rank: 1,
        },
        LeaderboardEntry {
            user: Pubkey::default(),
            score: 4500,
            category: ReputationCategory::Development,
            rank: 2,
        },
    ];

    // Calculate offset for pagination
    let offset = ReputationUtils::calculate_pagination_offset(page, page_size)?;
    let end_index = std::cmp::min(offset + page_size as usize, mock_entries.len());

    if offset < mock_entries.len() {
        leaderboard_entries.extend_from_slice(&mock_entries[offset..end_index]);
    }

    msg!("Leaderboard retrieved: page {}, size {}, entries: {}", page, page_size, leaderboard_entries.len());

    Ok(leaderboard_entries)
}

/// Calculate and update leaderboard rankings (admin function)
pub fn update_leaderboard_rankings(ctx: Context<UpdateLeaderboardRankings>) -> Result<()> {
    let config = &ctx.accounts.config;
    
    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // In a production system, this would:
    // 1. Query all user reputation accounts
    // 2. Sort by total score
    // 3. Update rankings
    // 4. Store top performers in season data

    msg!("Leaderboard rankings updated by admin: {}", ctx.accounts.admin.key());

    Ok(())
}

/// Get user's current ranking
pub fn get_user_ranking(ctx: Context<GetUserRanking>) -> Result<u32> {
    let user_reputation = &ctx.accounts.user_reputation;
    
    // This would calculate the user's actual rank by comparing with other users
    // For now, returning a mock rank based on score
    let mock_rank = if user_reputation.total_score > 5000 {
        1
    } else if user_reputation.total_score > 3000 {
        10
    } else if user_reputation.total_score > 1000 {
        50
    } else {
        100
    };

    msg!("User {} ranking: {}", user_reputation.user, mock_rank);

    Ok(mock_rank)
}

#[derive(Accounts)]
#[instruction(category: Option<ReputationCategory>, page: u32, page_size: u8)]
pub struct GetLeaderboard<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,
}

#[derive(Accounts)]
pub struct UpdateLeaderboardRankings<'info> {
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
pub struct GetUserRanking<'info> {
    #[account(
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    /// CHECK: User account being queried
    pub user: AccountInfo<'info>,
}