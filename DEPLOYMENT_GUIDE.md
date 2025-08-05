# Deployment Guide - DAO Reputation Scoreboard

This guide walks you through deploying the DAO Reputation Scoreboard to Solana networks.

## Prerequisites

- Node.js 16+ installed
- Rust and Cargo installed
- Solana CLI tools installed
- Anchor Framework 0.29.0+ installed
- A Solana wallet with sufficient SOL for deployment

## Environment Setup

### 1. Install Dependencies

```bash
# Install Node.js dependencies
npm install

# Install Anchor if not already installed
npm install -g @coral-xyz/anchor-cli@0.29.0
```

### 2. Configure Solana CLI

```bash
# Set cluster (devnet for testing, mainnet-beta for production)
solana config set --url https://api.devnet.solana.com

# Generate a new keypair if needed
solana-keygen new --outfile ~/.config/solana/id.json

# Check your wallet balance
solana balance
```

### 3. Configure Anchor

Update `Anchor.toml` with your target cluster:

```toml
[provider]
cluster = "Devnet"  # or "Mainnet" for production
wallet = "~/.config/solana/id.json"

[programs.devnet]
dao_reputation_scoreboard = "7ReputationDAOScoreboard11111111111111111111"
```

## Build and Deploy

### 1. Build the Program

```bash
anchor build
```

This generates:
- Program binary at `target/deploy/dao_reputation_scoreboard.so`
- IDL at `target/idl/dao_reputation_scoreboard.json`
- TypeScript types at `target/types/dao_reputation_scoreboard.ts`

### 2. Deploy to Devnet (Testing)

```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet
```

The deployment will output your program ID. Update this in your configuration files.

### 3. Deploy to Mainnet (Production)

⚠️ **IMPORTANT**: Thoroughly test on devnet before mainnet deployment!

```bash
# Switch to mainnet
solana config set --url https://api.mainnet-beta.solana.com

# Ensure you have sufficient SOL for deployment (~2-5 SOL)
solana balance

# Deploy to mainnet
anchor deploy --provider.cluster mainnet-beta
```

## Program Initialization

After deployment, initialize the reputation system:

### 1. Create Initialization Script

Create `scripts/initialize.ts`:

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DaoReputationScoreboard } from "../target/types/dao_reputation_scoreboard";
import { PublicKey, SystemProgram } from "@solana/web3.js";

async function initialize() {
  // Configure provider
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  
  const program = anchor.workspace.DaoReputationScoreboard as Program<DaoReputationScoreboard>;
  
  // Configuration parameters
  const VOTING_COOLDOWN = 600; // 10 minutes
  const MIN_ACCOUNT_AGE = 86400; // 1 day
  const DAILY_VOTE_LIMIT = 10;
  const MIN_REPUTATION_TO_VOTE = 100;
  const CATEGORY_WEIGHTS = [2500, 2500, 2500, 2500]; // Equal weights
  const ROLE_THRESHOLDS = [100, 500, 1000, 2500, 5000];
  
  // Derive config PDA
  const [configPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("reputation_config")],
    program.programId
  );
  
  try {
    await program.methods
      .initializeReputationSystem(
        new anchor.BN(VOTING_COOLDOWN),
        new anchor.BN(MIN_ACCOUNT_AGE),
        DAILY_VOTE_LIMIT,
        new anchor.BN(MIN_REPUTATION_TO_VOTE),
        CATEGORY_WEIGHTS,
        ROLE_THRESHOLDS.map(t => new anchor.BN(t))
      )
      .accounts({
        config: configPDA,
        admin: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    
    console.log("✅ Reputation system initialized successfully!");
    console.log("📍 Config PDA:", configPDA.toString());
    console.log("👑 Admin:", provider.wallet.publicKey.toString());
    
  } catch (error) {
    console.error("❌ Initialization failed:", error);
  }
}

initialize();
```

### 2. Run Initialization

```bash
npx ts-node scripts/initialize.ts
```

## Configuration Management

### Default Configuration

The system initializes with these default parameters:

```typescript
const DEFAULT_CONFIG = {
  votingCooldown: 600,        // 10 minutes
  minAccountAge: 86400,       // 1 day
  dailyVoteLimit: 10,         // 10 votes per day
  minReputationToVote: 100,   // 100 points minimum
  categoryWeights: [2500, 2500, 2500, 2500], // Equal 25% each
  roleThresholds: [100, 500, 1000, 2500, 5000], // Role unlock points
  decayRate: 10,              // 0.1% per day
  decayEnabled: true,         // Enable reputation decay
};
```

### Customization Examples

#### Conservative DAO (Higher barriers)
```typescript
const CONSERVATIVE_CONFIG = {
  votingCooldown: 3600,       // 1 hour cooldown
  minAccountAge: 604800,      // 7 days minimum age
  dailyVoteLimit: 5,          // 5 votes per day
  minReputationToVote: 500,   // 500 points to vote
  categoryWeights: [4000, 2000, 2000, 2000], // Governance focused
  roleThresholds: [500, 1000, 2500, 5000, 10000],
};
```

#### Active Community DAO (Lower barriers)
```typescript
const ACTIVE_CONFIG = {
  votingCooldown: 300,        // 5 minutes
  minAccountAge: 43200,       // 12 hours
  dailyVoteLimit: 20,         // 20 votes per day
  minReputationToVote: 50,    // 50 points to vote
  categoryWeights: [2000, 2000, 4000, 2000], // Community focused
  roleThresholds: [50, 200, 500, 1000, 2500],
};
```

#### Development-Focused DAO
```typescript
const DEV_CONFIG = {
  votingCooldown: 600,
  minAccountAge: 86400,
  dailyVoteLimit: 15,
  minReputationToVote: 100,
  categoryWeights: [2000, 4000, 2000, 2000], // Development focused
  roleThresholds: [100, 500, 1000, 2500, 5000],
};
```

## Post-Deployment Setup

### 1. Start First Season

```typescript
// Start initial competitive season
await program.methods
  .startNewSeason("Genesis Season", 90) // 90 days
  .accounts({
    config: configPDA,
    seasonData: seasonDataPDA,
    admin: adminKeypair.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([adminKeypair])
  .rpc();
```

### 2. Set Up Admin Operations

Create admin utility scripts for common operations:

```typescript
// scripts/admin-utils.ts
export class AdminUtils {
  constructor(private program: Program<DaoReputationScoreboard>) {}
  
  async updateConfig(newConfig: Partial<ReputationConfigUpdate>) {
    // Implementation for config updates
  }
  
  async bulkInitializeUsers(userPublicKeys: PublicKey[]) {
    // Implementation for bulk user initialization
  }
  
  async awardSeasonalBonuses(seasonWinners: PublicKey[]) {
    // Implementation for seasonal bonus distribution
  }
}
```

### 3. Set Up Monitoring

Implement monitoring for key metrics:

```typescript
// scripts/monitor.ts
export class ReputationMonitor {
  async getSystemStats() {
    // Total users, votes cast, top contributors, etc.
  }
  
  async checkSystemHealth() {
    // Verify configuration, check for abuse patterns
  }
  
  async generateReports() {
    // Generate periodic reports for DAO governance
  }
}
```

## Network-Specific Considerations

### Devnet Deployment
- Use for testing and development
- Faucet SOL available for testing
- Can reset/redeploy frequently
- Lower transaction fees

### Mainnet Deployment
- Production environment
- Real SOL required for transactions
- Immutable once deployed (unless upgradeable)
- Higher transaction fees
- Requires thorough testing

## Security Checklist

Before mainnet deployment, ensure:

- [ ] All tests pass on devnet
- [ ] Admin keys are properly secured
- [ ] Configuration parameters are validated
- [ ] Rate limiting is appropriate for your DAO size
- [ ] Emergency pause mechanisms tested
- [ ] Backup recovery procedures documented
- [ ] Multi-signature setup for admin operations (recommended)

## Cost Estimation

### Deployment Costs (Mainnet)
- Program deployment: ~2-5 SOL
- Account initialization: ~0.001 SOL per account
- Transaction fees: ~0.000005 SOL per transaction

### Ongoing Costs
- User reputation accounts: ~0.00144 SOL each
- Voting records: ~0.001 SOL each
- Seasonal data: ~0.002 SOL per season
- Admin operations: Standard transaction fees

## Troubleshooting

### Common Issues

#### "Program ID mismatch"
- Update `Anchor.toml` with deployed program ID
- Rebuild and regenerate types

#### "Insufficient funds"
- Ensure wallet has enough SOL
- Check network fees on mainnet

#### "Account already exists"
- Program already deployed at this address
- Use `anchor upgrade` instead of `anchor deploy`

#### "Invalid admin authority"
- Verify admin keypair in transactions
- Check configuration PDA derivation

### Recovery Procedures

#### Lost Admin Access
If admin keys are compromised:
1. Use emergency procedures if implemented
2. Contact Solana validators if necessary
3. Implement admin transfer procedures

#### Configuration Errors
1. Use admin update functions to correct
2. May require program upgrade for structural changes
3. Consider pause mechanisms for critical issues

## Upgrades and Migrations

### Program Upgrades
```bash
# Build new version
anchor build

# Upgrade program (requires upgrade authority)
anchor upgrade target/deploy/dao_reputation_scoreboard.so --program-id <PROGRAM_ID>
```

### Data Migration
For major version changes:
1. Export all user reputation data
2. Deploy new program version  
3. Migrate data using bulk operations
4. Verify data integrity

## Integration Examples

### Frontend Integration
```typescript
// React/TypeScript example
import { useAnchorWallet, useConnection } from '@solana/wallet-adapter-react';
import * as anchor from '@coral-xyz/anchor';

export function useReputationProgram() {
  const { connection } = useConnection();
  const wallet = useAnchorWallet();
  
  const program = useMemo(() => {
    if (!wallet) return null;
    
    const provider = new anchor.AnchorProvider(connection, wallet, {});
    return new anchor.Program(IDL, PROGRAM_ID, provider);
  }, [connection, wallet]);
  
  return program;
}
```

### Backend Integration
```typescript
// Node.js backend example
import { Connection, Keypair } from '@solana/web3.js';
import * as anchor from '@coral-xyz/anchor';

const connection = new Connection(process.env.SOLANA_RPC_URL);
const wallet = Keypair.fromSecretKey(
  Buffer.from(JSON.parse(process.env.ADMIN_PRIVATE_KEY))
);

const provider = new anchor.AnchorProvider(connection, wallet, {});
const program = new anchor.Program(IDL, PROGRAM_ID, provider);
```

## Support and Resources

- **Documentation**: See README.md for detailed API reference
- **Examples**: Check `examples/` directory
- **Tests**: Reference `tests/` for usage patterns
- **Issues**: Report bugs on GitHub
- **Community**: Join our Discord for support

---

**Note**: Always test thoroughly on devnet before mainnet deployment. This system handles valuable reputation data and should be deployed with care.