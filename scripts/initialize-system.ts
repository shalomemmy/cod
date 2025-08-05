#!/usr/bin/env ts-node

/**
 * System Initialization Script
 * 
 * This script initializes the DAO Reputation Scoreboard system with
 * configurable parameters for different DAO needs.
 */

import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DaoReputationScoreboard } from "../target/types/dao_reputation_scoreboard";
import { PublicKey, SystemProgram, Connection, Keypair } from "@solana/web3.js";
import * as fs from "fs";

// Configuration presets for different DAO types
const PRESETS = {
  conservative: {
    name: "Conservative DAO",
    votingCooldown: 3600,         // 1 hour
    minAccountAge: 604800,        // 7 days
    dailyVoteLimit: 5,            // 5 votes per day
    minReputationToVote: 500,     // 500 points to vote
    categoryWeights: [4000, 2000, 2000, 2000], // Governance focused
    roleThresholds: [500, 1000, 2500, 5000, 10000],
    decayRate: 5,                 // 0.05% per day
  },
  active: {
    name: "Active Community DAO",
    votingCooldown: 300,          // 5 minutes
    minAccountAge: 43200,         // 12 hours
    dailyVoteLimit: 20,           // 20 votes per day
    minReputationToVote: 50,      // 50 points to vote
    categoryWeights: [2000, 2000, 4000, 2000], // Community focused
    roleThresholds: [50, 200, 500, 1000, 2500],
    decayRate: 15,                // 0.15% per day
  },
  development: {
    name: "Development DAO",
    votingCooldown: 600,          // 10 minutes
    minAccountAge: 86400,         // 1 day
    dailyVoteLimit: 15,           // 15 votes per day
    minReputationToVote: 100,     // 100 points to vote
    categoryWeights: [2000, 4000, 2000, 2000], // Development focused
    roleThresholds: [100, 500, 1000, 2500, 5000],
    decayRate: 10,                // 0.1% per day
  },
  balanced: {
    name: "Balanced DAO",
    votingCooldown: 600,          // 10 minutes
    minAccountAge: 86400,         // 1 day
    dailyVoteLimit: 10,           // 10 votes per day
    minReputationToVote: 100,     // 100 points to vote
    categoryWeights: [2500, 2500, 2500, 2500], // Equal weights
    roleThresholds: [100, 500, 1000, 2500, 5000],
    decayRate: 10,                // 0.1% per day
  },
};

interface InitializationConfig {
  preset?: keyof typeof PRESETS;
  programId?: string;
  adminKeypairPath?: string;
  rpcUrl?: string;
  customConfig?: Partial<typeof PRESETS.balanced>;
}

class SystemInitializer {
  private program: Program<DaoReputationScoreboard>;
  private provider: anchor.AnchorProvider;
  private config: typeof PRESETS.balanced;

  constructor(
    connection: Connection,
    adminKeypair: Keypair,
    programId: PublicKey,
    config: typeof PRESETS.balanced
  ) {
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

    this.config = config;
  }

  async initialize(): Promise<void> {
    console.log(`🚀 Initializing ${this.config.name}...`);
    console.log("📋 Configuration:");
    console.log(`  Voting Cooldown: ${this.config.votingCooldown}s (${this.config.votingCooldown / 60} minutes)`);
    console.log(`  Min Account Age: ${this.config.minAccountAge}s (${this.config.minAccountAge / 86400} days)`);
    console.log(`  Daily Vote Limit: ${this.config.dailyVoteLimit}`);
    console.log(`  Min Reputation to Vote: ${this.config.minReputationToVote}`);
    console.log(`  Category Weights: [${this.config.categoryWeights.join(", ")}] (Governance, Development, Community, Treasury)`);
    console.log(`  Role Thresholds: [${this.config.roleThresholds.join(", ")}]`);
    console.log(`  Decay Rate: ${this.config.decayRate} basis points (${this.config.decayRate / 100}% per day)`);

    // Derive config PDA
    const [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("reputation_config")],
      this.program.programId
    );

    try {
      const tx = await this.program.methods
        .initializeReputationSystem(
          new anchor.BN(this.config.votingCooldown),
          new anchor.BN(this.config.minAccountAge),
          this.config.dailyVoteLimit,
          new anchor.BN(this.config.minReputationToVote),
          this.config.categoryWeights,
          this.config.roleThresholds.map(t => new anchor.BN(t))
        )
        .accounts({
          config: configPDA,
          admin: this.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log("✅ System initialized successfully!");
      console.log("📝 Transaction signature:", tx);
      console.log("📍 Config PDA:", configPDA.toString());
      console.log("👑 Admin:", this.provider.wallet.publicKey.toString());

      // Save configuration to file for reference  
      const configData = {
        programId: this.program.programId.toString(),
        configPDA: configPDA.toString(),
        admin: this.provider.wallet.publicKey.toString(),
        configuration: this.config,
        initializationTx: tx,
        timestamp: new Date().toISOString(),
      };

      fs.writeFileSync("reputation-system-config.json", JSON.stringify(configData, null, 2));
      console.log("💾 Configuration saved to reputation-system-config.json");

    } catch (error) {
      console.error("❌ Initialization failed:", error);
      throw error;
    }
  }

  async startInitialSeason(seasonId: number): Promise<void> {
    console.log("\n🏁 Starting initial season...");

    const [seasonDataPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("season_data"), new Uint8Array([seasonId, 0, 0, 0])], 
      this.program.programId
    );

    try {
      const tx = await this.program.methods
        .startNewSeason("Genesis Season", 90, seasonId) // 90 days
        .accounts({
          config: PublicKey.findProgramAddressSync(
            [Buffer.from("reputation_config")],
            this.program.programId
          )[0],
          seasonData: seasonDataPDA,
          admin: this.provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();

      console.log("✅ Genesis season started!");
      console.log("📝 Transaction signature:", tx);
      console.log("📍 Season Data PDA:", seasonDataPDA.toString());

    } catch (error) {
      console.error("❌ Season start failed:", error);
      throw error;
    }
  }

  async verifyInitialization(): Promise<void> {
    console.log("\n🔍 Verifying initialization...");

    const [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("reputation_config")],
      this.program.programId
    );

    try {
      const config = await this.program.account.reputationConfig.fetch(configPDA);
      
      console.log("✅ Configuration verified:");
      console.log(`  Admin: ${config.admin.toString()}`);
      console.log(`  Current Season: ${config.currentSeason}`);
      console.log(`  Total Users: ${config.totalUsers.toNumber()}`);
      console.log(`  Decay Enabled: ${config.decayEnabled}`);
      console.log(`  Initialized At: ${new Date(config.initializedAt.toNumber() * 1000).toISOString()}`);

    } catch (error) {
      console.error("❌ Verification failed:", error);
      throw error;
    }
  }
}

async function main() {
  const args = process.argv.slice(2);
  const preset = (args[0] as keyof typeof PRESETS) || "balanced";
  
  if (!PRESETS[preset]) {
    console.error(`❌ Invalid preset: ${preset}`);
    console.log("Available presets:", Object.keys(PRESETS).join(", "));
    process.exit(1);
  }

  // Configuration
  const config: InitializationConfig = {
    preset,
    programId: process.env.PROGRAM_ID,
    adminKeypairPath: process.env.ADMIN_KEYPAIR_PATH || "~/.config/solana/id.json",
    rpcUrl: process.env.RPC_URL || "https://api.devnet.solana.com",
  };

  console.log("🎯 DAO Reputation Scoreboard - System Initialization");
  console.log(`📦 Using preset: ${preset} (${PRESETS[preset].name})`);
  
  try {
    // Load program ID
    let programId: PublicKey;
    if (config.programId) {
      programId = new PublicKey(config.programId);
    } else {
      // Try to load from Anchor.toml or keypair file
      try {
        const keypairData = fs.readFileSync("target/deploy/dao_reputation_scoreboard-keypair.json", "utf8");
        const keypair = Keypair.fromSecretKey(new Uint8Array(JSON.parse(keypairData)));
        programId = keypair.publicKey;
      } catch {
        throw new Error("Program ID not found. Please deploy the program first or set PROGRAM_ID environment variable.");
      }
    }

    // Load admin keypair
    const adminKeypairPath = config.adminKeypairPath!.replace("~", process.env.HOME || "");
    const adminKeypairData = fs.readFileSync(adminKeypairPath, "utf8");
    const adminKeypair = Keypair.fromSecretKey(new Uint8Array(JSON.parse(adminKeypairData)));

    // Create connection
    const connection = new Connection(config.rpcUrl!, "confirmed");

    // Check balance
    const balance = await connection.getBalance(adminKeypair.publicKey);
    console.log(`💰 Admin balance: ${balance / 1e9} SOL`);
    
    if (balance < 0.1 * 1e9) { // Less than 0.1 SOL
      console.warn("⚠️  Low balance. You may need more SOL for initialization.");
    }

    // Initialize system
    const selectedConfig = { ...PRESETS[preset], ...config.customConfig };
    const initializer = new SystemInitializer(connection, adminKeypair, programId, selectedConfig);
    
    await initializer.initialize();
    await initializer.startInitialSeason(1);
    await initializer.verifyInitialization();

    console.log("\n🎉 System initialization completed successfully!");
    console.log("\n📚 Next steps:");
    console.log("  1. Initialize user reputation accounts for initial members");
    console.log("  2. Set up monitoring and analytics");
    console.log("  3. Configure your frontend/backend integration");
    console.log("  4. Begin community onboarding");

  } catch (error) {
    console.error("\n❌ Initialization failed:", error);
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}

export { SystemInitializer, PRESETS };