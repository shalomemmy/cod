# DAO Reputation Scoreboard

A comprehensive reputation tracking system for Solana DAOs with multi-dimensional scoring, gamification, and anti-abuse mechanisms.

## 🌟 Features

### Core Functionality
- **Multi-dimensional reputation scoring** across 4 categories: Governance, Development, Community, Treasury
- **Token-gated voting system** with configurable cooldown periods and daily limits
- **Quadratic reputation formula** to prevent whale dominance
- **Role-based unlocks** for top contributors based on reputation thresholds
- **Comprehensive admin controls** for system management

### Gamification & Engagement
- **Achievement badge system** with 8 different achievement types
- **Streak bonuses** for consistent participation
- **Seasonal competitions** with automatic leaderboard resets
- **Reputation decay system** to encourage ongoing participation

### Anti-Abuse Mechanisms
- **Minimum account age** requirements before voting
- **Daily vote limits** per wallet
- **Voting cooldowns** between votes from same wallet
- **Minimum reputation requirements** for voting on others
- **Quadratic scaling** to prevent gaming by large token holders

### Advanced Features
- **Reputation certificates** for portable reputation export
- **Bulk admin operations** for efficient management
- **Configurable parameters** for different DAO needs
- **Event emission** for external integrations
- **Comprehensive error handling** with custom error types

## 🏗️ Architecture

### Account Structure

1. **ReputationConfig** - Global system settings
   - Admin authority and system parameters
   - Category weights and role thresholds
   - Season information and decay settings

2. **UserReputation** - Individual user scores
   - Points across 4 reputation categories
   - Achievement badges and streak information
   - Role level and activity timestamps

3. **VotingRecord** - Anti-abuse tracking
   - Voting history with timestamps
   - Daily vote counts and cooldown tracking
   - Target-specific vote limitations

4. **SeasonData** - Competition periods
   - Season leaderboards and statistics
   - Participation tracking and rewards

### Key Instructions

- `initialize_reputation_system` - Set up program with admin config
- `cast_vote` - Upvote/downvote with anti-abuse checks
- `update_user_reputation` - Admin function for manual adjustments
- `get_leaderboard` - Paginated leaderboard retrieval
- `claim_role_unlock` - Role claims based on reputation thresholds
- `start_new_season` - Admin function for seasonal competitions
- `export_reputation` - Generate portable reputation certificates

## 🚀 Quick Start

### Prerequisites
- Node.js 16+
- Rust and Cargo
- Solana CLI tools
- Anchor Framework 0.29.0+

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd dao-reputation-scoreboard
```

2. Install dependencies:
```bash
npm install
```

3. Build the program:
```bash
anchor build
```

4. Run tests:
```bash
anchor test
```

### Deployment

1. Configure your Solana cluster in `Anchor.toml`
2. Deploy the program:
```bash
anchor deploy
```

3. Initialize the reputation system:
```typescript
await program.methods
  .initializeReputationSystem(
    new BN(600),        // 10 minute voting cooldown
    new BN(86400),      // 1 day minimum account age
    10,                 // 10 votes per day limit
    new BN(100),        // 100 points minimum to vote
    [2500, 2500, 2500, 2500], // Equal category weights
    [100, 500, 1000, 2500, 5000] // Role thresholds
  )
  .accounts({
    config: configPDA,
    admin: adminKeypair.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([adminKeypair])
  .rpc();
```

## 📊 Usage Examples

### Initialize User Reputation
```typescript
await program.methods
  .initializeUserReputation()
  .accounts({
    config: configPDA,
    userReputation: userReputationPDA,
    user: userKeypair.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([userKeypair])
  .rpc();
```

### Cast Vote
```typescript
await program.methods
  .castVote(
    true,                    // isUpvote
    { governance: {} },      // category
    5                        // vote weight (1-10)
  )
  .accounts({
    config: configPDA,
    voterReputation: voterReputationPDA,
    targetReputation: targetReputationPDA,
    votingRecord: votingRecordPDA,
    voter: voterKeypair.publicKey,
    target: targetPublicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([voterKeypair])
  .rpc();
```

### Claim Role Unlock
```typescript
await program.methods
  .claimRoleUnlock(2) // Role level
  .accounts({
    config: configPDA,
    userReputation: userReputationPDA,
    user: userKeypair.publicKey,
  })
  .signers([userKeypair])
  .rpc();
```

### Get Leaderboard
```typescript
const leaderboard = await program.methods
  .getLeaderboard(
    { governance: {} }, // category filter (optional)
    0,                  // page number
    10                  // page size
  )
  .accounts({
    config: configPDA,
  })
  .view();
```

### Export Reputation Certificate
```typescript
const certificate = await program.methods
  .exportReputation()
  .accounts({
    userReputation: userReputationPDA,
    user: userPublicKey,
  })
  .view();
```

## 🔧 Configuration

### Category Weights
Configure how much each category contributes to total score:
```typescript
const categoryWeights = [
  3000, // Governance (30%)
  2500, // Development (25%)
  2500, // Community (25%)
  2000  // Treasury (20%)
];
```

### Role Thresholds
Set reputation requirements for role unlocks:
```typescript
const roleThresholds = [
  100,   // Role 1: Contributor
  500,   // Role 2: Active Member
  1000,  // Role 3: Senior Member
  2500,  // Role 4: Core Member
  5000   // Role 5: Leadership
];
```

### Anti-Abuse Settings
```typescript
const antiAbuseConfig = {
  votingCooldown: 600,        // 10 minutes between votes
  minAccountAge: 86400,       // 1 day minimum account age
  dailyVoteLimit: 10,         // 10 votes per day
  minReputationToVote: 100,   // 100 points to vote on others
};
```

## 🎯 Achievement System

### Available Achievements
- **FirstVote** - Cast your first vote
- **WeeklyStreak** - 7 consecutive days of activity
- **MonthlyStreak** - 30 consecutive days of activity
- **TopContributor** - Reach 10,000 total reputation
- **ConsistentVoter** - Cast 100+ votes
- **CategoryExpert** - Reach 5,000 points in any category
- **SeasonWinner** - Win a seasonal competition
- **CommunityBuilder** - Reach 3,000 community points

### Auto-Award Achievements
```typescript
await program.methods
  .autoAwardAchievements()
  .accounts({
    userReputation: userReputationPDA,
    user: userPublicKey,
  })
  .rpc();
```

## 📈 Seasonal Competitions

### Start New Season
```typescript
await program.methods
  .startNewSeason(
    "Spring 2024 Competition", // season name
    90                         // duration in days
  )
  .accounts({
    config: configPDA,
    seasonData: seasonDataPDA,
    admin: adminKeypair.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([adminKeypair])
  .rpc();
```

### Reset Seasonal Points
```typescript
await program.methods
  .resetSeasonalPoints()
  .accounts({
    config: configPDA,
    userReputation: userReputationPDA,
    admin: adminKeypair.publicKey,
    user: userPublicKey,
  })
  .signers([adminKeypair])
  .rpc();
```

## 🛡️ Security Features

### Quadratic Scaling
Reputation gains use quadratic scaling to prevent whale dominance:
```
reputation_gain = sqrt(raw_votes) * category_weight
```

### Voting Limits
- Maximum 10 votes per day per wallet
- Minimum 10-minute cooldown between votes
- Minimum 1-day account age requirement
- Minimum reputation threshold to vote on others

### Admin Controls
- Update system configuration
- Manual reputation adjustments
- Bulk operations for management
- Emergency pause functionality
- Admin authority transfer

## 🔍 Testing

The project includes comprehensive tests covering:

1. **System Initialization** - Configuration validation and setup
2. **Voting Mechanics** - All voting scenarios and edge cases
3. **Reputation Calculation** - Score accuracy across categories
4. **Leaderboard Generation** - Pagination and filtering
5. **Role Unlock Functionality** - Threshold validation
6. **Anti-Abuse Mechanisms** - Cooldowns, limits, and restrictions
7. **Admin Operations** - Permission checks and bulk operations
8. **Edge Cases** - Error handling and data validation
9. **Seasonal Functionality** - Competition management
10. **Export Operations** - Certificate generation and verification

Run the full test suite:
```bash
anchor test
```

Run specific test categories:
```bash
anchor test --grep "Voting Mechanics"
anchor test --grep "Anti-Abuse"
anchor test --grep "Achievement System"
```

## 📚 API Reference

### Core Instructions

#### `initialize_reputation_system`
Initialize the reputation system with admin configuration.

**Parameters:**
- `voting_cooldown: u64` - Cooldown between votes (seconds)
- `min_account_age: u64` - Minimum account age (seconds)
- `daily_vote_limit: u8` - Maximum votes per day
- `min_reputation_to_vote: u64` - Minimum reputation to vote
- `category_weights: [u16; 4]` - Weight for each category (basis points)
- `role_thresholds: [u64; 5]` - Reputation thresholds for roles

#### `cast_vote`
Cast an upvote or downvote on another user's reputation.

**Parameters:**
- `is_upvote: bool` - Whether this is an upvote
- `category: ReputationCategory` - Which category to vote on
- `vote_weight: u8` - Weight of the vote (1-10)

#### `claim_role_unlock`
Claim a role unlock based on reputation thresholds.

**Parameters:**
- `role_level: u8` - Role level to unlock (1-5)

### View Instructions

#### `get_leaderboard`
Get paginated leaderboard data.

**Returns:** `Vec<LeaderboardEntry>`

#### `export_reputation`
Export user reputation as a portable certificate.

**Returns:** `ReputationCertificate`

#### `get_user_streak_info`
Get detailed streak information for a user.

**Returns:** `StreakInfo`

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

### Code Standards
- Follow Rust formatting standards
- Add comprehensive comments
- Include error handling
- Write unit tests for new features
- Update documentation

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

For support, please:
1. Check the documentation
2. Search existing issues
3. Create a new issue with detailed information
4. Join our Discord community

## 🗺️ Roadmap

### Version 1.1
- [ ] Cross-chain reputation bridging
- [ ] Advanced analytics dashboard
- [ ] Mobile-friendly interfaces
- [ ] NFT integration for achievements

### Version 1.2
- [ ] Governance proposal integration
- [ ] Automated reward distribution
- [ ] Social features and networking
- [ ] Advanced reporting tools

## 👥 Team

Built with ❤️ by the DAO Reputation team for the Solana ecosystem.

---

**Note:** This is a production-ready template that Solana DAOs can fork and customize for their specific needs. The focus is on security, efficiency, and developer experience.# cod
