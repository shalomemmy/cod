/**
 * Basic Usage Example - DAO Reputation Scoreboard
 * 
 * This example demonstrates the core functionality of the DAO Reputation Scoreboard,
 * including initialization, user management, voting, and reputation tracking.
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DaoReputationScoreboard } from "../target/types/dao_reputation_scoreboard";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram, 
  Connection,
  LAMPORTS_PER_SOL 
} from "@solana/web3.js";

// Example configuration for a small DAO
const DAO_CONFIG = {
  votingCooldown: 600,        // 10 minutes between votes
  minAccountAge: 86400,       // 1 day minimum account age
  dailyVoteLimit: 10,         // 10 votes per day maximum
  minReputationToVote: 100,   // 100 points minimum to vote
  categoryWeights: [2500, 2500, 2500, 2500], // Equal 25% weights
  roleThresholds: [100, 500, 1000, 2500, 5000], // Role unlock thresholds
};

export class ReputationManager {
  private program: Program<DaoReputationScoreboard>;
  private provider: anchor.AnchorProvider;
  private configPDA: PublicKey;

  constructor(
    connection: Connection, 
    adminKeypair: Keypair, 
    programId: PublicKey
  ) {
    // Initialize Anchor provider and program
    this.provider = new anchor.AnchorProvider(
      connection,
      new anchor.Wallet(adminKeypair),
      { commitment: "confirmed" }
    );
    
    anchor.setProvider(this.provider);
    this.program = new Program(
      require("../target/idl/dao_reputation_scoreboard.json"),
      programId,
      this.provider
    );

    // Derive config PDA
    [this.configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("reputation_config")],
      this.program.programId
    );
  }

  /**
   * Initialize the reputation system (one-time setup)
   */
  async initializeSystem(): Promise<void> {
    console.log("🚀 Initializing DAO Reputation System...");

    try {
      const tx = await this.program.methods
        .initializeReputationSystem(
          new anchor.BN(DAO_CONFIG.votingCooldown),
          new anchor.BN(DAO_CONFIG.minAccountAge),
          DAO_CONFIG.dailyVoteLimit,
          new anchor.BN(DAO_CONFIG.minReputationToVote),
          DAO_CONFIG.categoryWeights,
          DAO_CONFIG.roleThresholds.map(t => new anchor.BN(t))
        )
        .accounts({
          config: this.configPDA,
          admin: this.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log("✅ System initialized successfully!");
      console.log("📝 Transaction:", tx);
      console.log("📍 Config PDA:", this.configPDA.toString());

    } catch (error) {
      console.error("❌ Initialization failed:", error);
      throw error;
    }
  }

  /**
   * Initialize a new user's reputation account
   */
  async initializeUser(userKeypair: Keypair): Promise<PublicKey> {
    console.log(`👤 Initializing user: ${userKeypair.publicKey.toString()}`);

    const [userReputationPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_reputation"), userKeypair.publicKey.toBuffer()],
      this.program.programId
    );

    try {
      const tx = await this.program.methods
        .initializeUserReputation()
        .accounts({
          config: this.configPDA,
          userReputation: userReputationPDA,
          user: userKeypair.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([userKeypair])
        .rpc();

      console.log("✅ User initialized successfully!");
      console.log("📝 Transaction:", tx);
      console.log("📍 User Reputation PDA:", userReputationPDA.toString());

      return userReputationPDA;

    } catch (error) {
      console.error("❌ User initialization failed:", error);
      throw error;
    }
  }

  /**
   * Cast a vote on another user's reputation
   */
  async castVote(
    voterKeypair: Keypair,
    targetPublicKey: PublicKey,
    isUpvote: boolean,
    category: "governance" | "development" | "community" | "treasury",
    weight: number
  ): Promise<void> {
    console.log(`🗳️  ${voterKeypair.publicKey.toString()} voting on ${targetPublicKey.toString()}`);

    // Derive necessary PDAs
    const [voterReputationPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_reputation"), voterKeypair.publicKey.toBuffer()],
      this.program.programId
    );

    const [targetReputationPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_reputation"), targetPublicKey.toBuffer()],
      this.program.programId
    );

    const [votingRecordPDA] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("voting_record"),
        voterKeypair.publicKey.toBuffer(),
        targetPublicKey.toBuffer()
      ],
      this.program.programId
    );

    // Convert category string to enum object
    const categoryEnum = { [category]: {} };

    try {
      const tx = await this.program.methods
        .castVote(isUpvote, categoryEnum, weight)
        .accounts({
          config: this.configPDA,
          voterReputation: voterReputationPDA,
          targetReputation: targetReputationPDA,
          votingRecord: votingRecordPDA,
          voter: voterKeypair.publicKey,
          target: targetPublicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voterKeypair])
        .rpc();

      console.log(`✅ Vote cast successfully! ${isUpvote ? '👍' : '👎'}`);
      console.log("📝 Transaction:", tx);

    } catch (error) {
      console.error("❌ Vote failed:", error);
      throw error;
    }
  }

  /**
   * Get user's reputation data
   */
  async getUserReputation(userPublicKey: PublicKey): Promise<any> {
    const [userReputationPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_reputation"), userPublicKey.toBuffer()],
      this.program.programId
    );

    try {
      const reputation = await this.program.account.userReputation.fetch(userReputationPDA);
      return {
        user: reputation.user.toString(),
        totalScore: reputation.totalScore.toNumber(),
        categoryPoints: reputation.categoryPoints.map(p => p.toNumber()),
        roleLevel: reputation.roleLevel,
        achievements: reputation.achievements.toString(2), // Binary representation
        currentStreak: reputation.currentStreak,
        longestStreak: reputation.longestStreak,
        votesCast: reputation.votesCast.toNumber(),
        lastActivity: new Date(reputation.lastActivity.toNumber() * 1000).toISOString(),
      };
    } catch (error) {
      console.error("❌ Failed to fetch user reputation:", error);
      return null;
    }
  }

  /**
   * Get system configuration
   */
  async getSystemConfig(): Promise<any> {
    try {
      const config = await this.program.account.reputationConfig.fetch(this.configPDA);
      return {
        admin: config.admin.toString(),
        votingCooldown: config.votingCooldown.toNumber(),
        minAccountAge: config.minAccountAge.toNumber(),
        dailyVoteLimit: config.dailyVoteLimit,
        minReputationToVote: config.minReputationToVote.toNumber(),
        categoryWeights: config.categoryWeights,
        roleThresholds: config.roleThresholds.map(t => t.toNumber()),
        currentSeason: config.currentSeason,
        totalUsers: config.totalUsers.toNumber(),
        decayEnabled: config.decayEnabled,
      };
    } catch (error) {
      console.error("❌ Failed to fetch system config:", error);
      return null;
    }
  }

  /**
   * Claim role unlock based on reputation
   */
  async claimRoleUnlock(userKeypair: Keypair, roleLevel: number): Promise<void> {
    console.log(`🏆 ${userKeypair.publicKey.toString()} claiming role level ${roleLevel}`);

    const [userReputationPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_reputation"), userKeypair.publicKey.toBuffer()],
      this.program.programId
    );

    try {
      const tx = await this.program.methods
        .claimRoleUnlock(roleLevel)
        .accounts({
          config: this.configPDA,
          userReputation: userReputationPDA,
          user: userKeypair.publicKey,
        })
        .signers([userKeypair])
        .rpc();

      console.log("✅ Role unlocked successfully!");
      console.log("📝 Transaction:", tx);

    } catch (error) {
      console.error("❌ Role unlock failed:", error);
      throw error;
    }
  }

  /**
   * Export user's reputation certificate
   */
  async exportReputationCertificate(userPublicKey: PublicKey): Promise<any> {
    const [userReputationPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_reputation"), userPublicKey.toBuffer()],
      this.program.programId
    );

    try {
      const certificate = await this.program.methods
        .exportReputation()
        .accounts({
          userReputation: userReputationPDA,
          user: userPublicKey,
        })
        .view();

      console.log("📜 Reputation certificate exported successfully!");
      return {
        user: certificate.user.toString(),
        totalScore: certificate.totalScore.toNumber(),
        categoryScores: certificate.categoryScores.map(s => s.toNumber()),
        achievements: certificate.achievements.toString(2),
        roleLevel: certificate.roleLevel,
        generatedAt: new Date(certificate.generatedAt.toNumber() * 1000).toISOString(),
        programId: certificate.programId.toString(),
        signatureHash: Array.from(certificate.signatureHash),
      };

    } catch (error) {
      console.error("❌ Certificate export failed:", error);
      return null;
    }
  }

  /**
   * Start a new competitive season (admin only)
   */
  async startNewSeason(seasonName: string, durationDays: number, seasonId: number): Promise<void> {
    console.log(`🏁 Starting new season: ${seasonName} (${durationDays} days)`);

    const [seasonDataPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("season_data"), new Uint8Array([seasonId, 0, 0, 0])],
      this.program.programId
    );

    try {
      const tx = await this.program.methods
        .startNewSeason(seasonName, durationDays, seasonId)
        .accounts({
          config: this.configPDA,
          seasonData: seasonDataPDA,
          admin: this.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log("✅ New season started successfully!");
      console.log("📝 Transaction:", tx);
      console.log("📍 Season Data PDA:", seasonDataPDA.toString());

    } catch (error) {
      console.error("❌ Season start failed:", error);
      throw error;
    }
  }

  /**
   * Get leaderboard data
   */
  async getLeaderboard(
    category?: "governance" | "development" | "community" | "treasury",
    page: number = 0,
    pageSize: number = 10
  ): Promise<any[]> {
    try {
      const categoryEnum = category ? { [category]: {} } : null;
      
      const leaderboard = await this.program.methods
        .getLeaderboard(categoryEnum, page, pageSize)
        .accounts({
          config: this.configPDA,
        })
        .view();

      return leaderboard.map((entry: any) => ({
        user: entry.user.toString(),
        totalScore: entry.totalScore.toNumber(),
        categoryScores: entry.categoryScores.map((s: any) => s.toNumber()),
        rank: entry.rank,
      }));

    } catch (error) {
      console.error("❌ Failed to fetch leaderboard:", error);
      return [];
    }
  }
}

/**
 * Complete example usage
 */
async function runExample() {
  console.log("🎯 DAO Reputation Scoreboard - Basic Usage Example\n");

  // Setup connection and keypairs
  const connection = new Connection("http://localhost:8899", "confirmed");
  const adminKeypair = Keypair.generate();
  const user1 = Keypair.generate();
  const user2 = Keypair.generate();
  const programId = new PublicKey("7ReputationDAOScoreboard11111111111111111111");

  // Airdrop SOL for testing
  console.log("💰 Airdropping SOL for testing...");
  await connection.requestAirdrop(adminKeypair.publicKey, 2 * LAMPORTS_PER_SOL);
  await connection.requestAirdrop(user1.publicKey, LAMPORTS_PER_SOL);
  await connection.requestAirdrop(user2.publicKey, LAMPORTS_PER_SOL);

  // Wait for airdrops
  await new Promise(resolve => setTimeout(resolve, 2000));

  // Initialize reputation manager
  const reputationManager = new ReputationManager(connection, adminKeypair, programId);

  try {
    // 1. Initialize the system
    await reputationManager.initializeSystem();

    // 2. Initialize users
    console.log("\n👥 Initializing users...");
    await reputationManager.initializeUser(user1);
    await reputationManager.initializeUser(user2);

    // 3. Give user1 initial reputation to meet voting requirements
    console.log("\n⚡ Giving initial reputation to user1...");
    // This would be done through admin functions or initial contributions

    // 4. Cast some votes
    console.log("\n🗳️  Casting votes...");
    
    // Wait for account age requirement (in real scenario)
    console.log("⏳ Simulating account age requirement...");
    
    try {
      await reputationManager.castVote(user1, user2.publicKey, true, "governance", 7);
      await new Promise(resolve => setTimeout(resolve, 1000));
      await reputationManager.castVote(user1, user2.publicKey, true, "community", 5);
    } catch (error) {
      console.log("Expected error due to account age/reputation requirements:", error.message);
    }

    // 5. Check user reputations
    console.log("\n📊 Checking user reputations...");
    const user1Rep = await reputationManager.getUserReputation(user1.publicKey);
    const user2Rep = await reputationManager.getUserReputation(user2.publicKey);
    
    console.log("User 1 Reputation:", user1Rep);
    console.log("User 2 Reputation:", user2Rep);

    // 6. Get leaderboard
    console.log("\n🏆 Getting leaderboard...");
    const leaderboard = await reputationManager.getLeaderboard();
    console.log("Leaderboard:", leaderboard);

    // 7. Export reputation certificate
    console.log("\n📜 Exporting reputation certificate...");
    const certificate = await reputationManager.exportReputationCertificate(user1.publicKey);
    console.log("Certificate:", certificate);

    // 8. Start new season
    console.log("\n🏁 Starting new season...");
    await reputationManager.startNewSeason("Example Season", 30, 2);

    // 9. Get system configuration
    console.log("\n⚙️  System configuration:");
    const config = await reputationManager.getSystemConfig();
    console.log(config);

    console.log("\n✅ Example completed successfully!");

  } catch (error) {
    console.error("\n❌ Example failed:", error);
  }
}

// Run example if this file is executed directly
if (require.main === module) {
  runExample().catch(console.error);
}

export { ReputationManager, DAO_CONFIG, runExample };