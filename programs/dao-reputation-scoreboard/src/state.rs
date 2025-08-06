/// Streak information
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
}