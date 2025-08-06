/// Calculate potential decay without applying it
pub fn calculate_decay_preview(
    ctx: Context<CalculateDecayPreview>,
    user: Pubkey,
) -> Result<DecayPreview> {
    let config = &ctx.accounts.config;
    let user_reputation = &ctx.accounts.user_reputation;
    let current_time = ReputationUtils::get_current_timestamp();

    let days_inactive = if user_reputation.last_activity > 0 {
        (current_time - user_reputation.last_activity) / 86400
    } else {
        0
    };

    let decay_factor = if days_inactive > 0 && config.decay_enabled {
        ReputationUtils::calculate_decay_factor(
            user_reputation.last_activity,
            current_time,
            config.decay_rate,
        )
    } else {
        10000 // No decay
    };

    let mut projected_points = [0u64; 4];
    // Removed unused total_decay variable

    for (i, &points) in user_reputation.category_points.iter().enumerate() {
        projected_points[i] = (points * decay_factor) / 10000;
        // Removed total_decay calculation
    }

    // Calculate projected total score
    let mut projected_total_score = 0u64;
    for (i, &points) in projected_points.iter().enumerate() {
        let scaled_points = if points == 0 {
            0
        } else {
            ReputationUtils::calculate_quadratic_weight(points)
        };
        projected_total_score += scaled_points * (config.category_weights[i] as u64);
    }
    projected_total_score /= 10000;

    // Removed unused projected_role_level variable

    // Calculate decay amounts for preview
    let mut decay_amounts = [0u64; 4];
    let mut new_category_points = user_reputation.category_points;
    
    if config.decay_enabled && days_inactive >= 7 {
        let decay_factor = 9500; // 5% decay = 95% remaining
        for i in 0..4 {
            let decay_amount = (user_reputation.category_points[i] * (10000 - decay_factor)) / 10000;
            decay_amounts[i] = decay_amount;
            new_category_points[i] = user_reputation.category_points[i] - decay_amount;
        }
    }

    let preview = DecayPreview {
        current_points: user_reputation.category_points,
        points_after_decay: new_category_points,
        decay_amount: decay_amounts,
        days_since_activity: days_inactive as u64,
        will_decay: config.decay_enabled && days_inactive >= 7,
    };

    Ok(preview)
}