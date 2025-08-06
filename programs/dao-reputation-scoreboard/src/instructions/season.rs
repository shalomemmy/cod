use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;
use crate::utils::*;

/// Admin function to start a new seasonal competition
pub fn start_new_season(
    ctx: Context<StartNewSeason>,
    season_name: String,
    duration_days: u32,
    season_id: u32,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let season_data = &mut ctx.accounts.season_data;
    let current_time = ReputationUtils::get_current_timestamp();

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Validate season parameters
    ReputationUtils::validate_string_length(&season_name, 50)?;
    require!(duration_days > 0 && duration_days <= 365, ReputationError::SeasonDurationTooLong);
    require!(season_id > config.current_season, ReputationError::InvalidConfigurationValues);

    // Update config for new season
    config.current_season = season_id;
    config.season_start = current_time;
    config.season_duration = (duration_days as u64) * 86400; // Convert days to seconds

    // Initialize season data
    season_data.season_id = season_id;
    season_data.start_time = current_time;
    season_data.end_time = current_time + config.season_duration as i64;
    season_data.is_active = true;
    season_data.leaderboard = [LeaderboardEntry::default(); 1];
    season_data.total_participants = 0;
    season_data.rewards_distributed = false;
    season_data.total_votes_cast = 0;
    season_data.most_active_category = ReputationCategory::Governance;
    season_data.reserved = [0; 2];

    msg!(
        "New season started: {} (ID: {}, Duration: {} days)",
        season_name,
        season_id,
        duration_days
    );

    Ok(())
}

/// End current season and distribute rewards
pub fn end_current_season(ctx: Context<EndCurrentSeason>, _season_id: u32) -> Result<()> {
    let config = &mut ctx.accounts.config;
    let season_data = &mut ctx.accounts.season_data;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Check if season is currently active
    require!(season_data.is_active, ReputationError::SeasonNotActive);

    // Mark season as ended
    season_data.is_active = false;
    season_data.end_time = ReputationUtils::get_current_timestamp();

    // Award seasonal achievements to top performers
    // This would typically involve iterating through all users and awarding bonuses

    msg!("Season {} ended", season_data.season_id);

    Ok(())
}

/// Get current season information
pub fn get_season_info(ctx: Context<GetSeasonInfo>, _season_id: u32) -> Result<SeasonInfo> {
    let _config = &ctx.accounts.config;
    let season_data = &ctx.accounts.season_data;
    let current_time = ReputationUtils::get_current_timestamp();

    let season_info = SeasonInfo {
        season_id: season_data.season_id,
        name: format!("Season {}", season_data.season_id),
        start_time: season_data.start_time,
        end_time: season_data.end_time,
        is_active: season_data.is_active && current_time < season_data.end_time,
        total_participants: season_data.total_participants,
        total_votes: season_data.total_votes_cast,
        days_remaining: if season_data.is_active && current_time < season_data.end_time {
            ((season_data.end_time - current_time) / 86400) as u64
        } else {
            0
        },
    };

    Ok(season_info)
}

/// Reset user seasonal points (called when new season starts)
pub fn reset_seasonal_points(ctx: Context<ResetSeasonalPoints>) -> Result<()> {
    let config = &ctx.accounts.config;
    let user_reputation = &mut ctx.accounts.user_reputation;

    // Validate admin authority
    require!(
        ctx.accounts.admin.key() == config.admin,
        ReputationError::UnauthorizedAdmin
    );

    // Store previous season best rank if better than current
    // This would be calculated from actual leaderboard data
    let current_rank = 100; // Mock rank calculation
    if user_reputation.best_season_rank == 0 || current_rank < user_reputation.best_season_rank {
        user_reputation.best_season_rank = current_rank;
    }

    // Reset seasonal points
    user_reputation.seasonal_points = [0; 4];

    msg!("Seasonal points reset for user: {}", user_reputation.user);

    Ok(())
}

// SeasonInfo is now defined in state.rs - removed duplicate

#[derive(Accounts)]
#[instruction(season_name: String, duration_days: u32, season_id: u32)]
pub struct StartNewSeason<'info> {
    #[account(
        mut,
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        init,
        payer = admin,
        space = SeasonData::LEN,
        seeds = [b"season_data", &season_id.to_le_bytes()[..]], // Fixed: Use slice to handle array size
        bump
    )]
    pub season_data: Account<'info, SeasonData>,

    #[account(
        mut,
        constraint = admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(season_id: u32)]
pub struct EndCurrentSeason<'info> {
    #[account(
        mut,
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        mut,
        seeds = [b"season_data", &season_id.to_le_bytes()[..]], // Fixed: Use slice to handle array size
        bump
    )]
    pub season_data: Account<'info, SeasonData>,

    #[account(
        constraint = admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(season_id: u32)]
pub struct GetSeasonInfo<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        seeds = [b"season_data", &season_id.to_le_bytes()[..]], // Fixed: Use slice to handle array size
        bump
    )]
    pub season_data: Account<'info, SeasonData>,
}

#[derive(Accounts)]
pub struct ResetSeasonalPoints<'info> {
    #[account(
        seeds = [b"reputation_config"],
        bump
    )]
    pub config: Account<'info, ReputationConfig>,

    #[account(
        mut,
        seeds = [b"user_reputation", user.key().as_ref()],
        bump
    )]
    pub user_reputation: Account<'info, UserReputation>,

    #[account(
        constraint = admin.key() == config.admin @ ReputationError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// CHECK: User account being reset
    pub user: AccountInfo<'info>,
}