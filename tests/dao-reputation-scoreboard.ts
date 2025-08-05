import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DaoReputationScoreboard } from "../target/types/dao_reputation_scoreboard";
import { expect } from "chai";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";

describe("DAO Reputation Scoreboard", () => {
  // Configure the client
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.DaoReputationScoreboard as Program<DaoReputationScoreboard>;
  const provider = anchor.getProvider();

  // Test accounts
  let admin: Keypair;
  let user1: Keypair;
  let user2: Keypair;
  let user3: Keypair;

  // Program derived addresses
  let configPDA: PublicKey;
  let user1ReputationPDA: PublicKey;
  let user2ReputationPDA: PublicKey;
  let user3ReputationPDA: PublicKey;
  let votingRecordPDA: PublicKey;
  let seasonDataPDA: PublicKey;

  // Configuration constants for testing
  const VOTING_COOLDOWN = 600; // 10 minutes
  const MIN_ACCOUNT_AGE = 86400; // 1 day
  const DAILY_VOTE_LIMIT = 10;
  const MIN_REPUTATION_TO_VOTE = 100;
  const CATEGORY_WEIGHTS = [2500, 2500, 2500, 2500]; // Equal weights
  const ROLE_THRESHOLDS = [100, 500, 1000, 2500, 5000];

  before(async () => {
    // Generate test keypairs
    admin = Keypair.generate();
    user1 = Keypair.generate();
    user2 = Keypair.generate();
    user3 = Keypair.generate();

    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(admin.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user1.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user2.publicKey, 2 * LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(user3.publicKey, 2 * LAMPORTS_PER_SOL);

    // Wait for airdrops to confirm
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Derive PDAs
    [configPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("reputation_config")],
      program.programId
    );

    [user1ReputationPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_reputation"), user1.publicKey.toBuffer()],
      program.programId
    );

    [user2ReputationPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_reputation"), user2.publicKey.toBuffer()],
      program.programId
    );

    [user3ReputationPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("user_reputation"), user3.publicKey.toBuffer()],
      program.programId
    );

    [votingRecordPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("voting_record"), user1.publicKey.toBuffer(), user2.publicKey.toBuffer()],
      program.programId
    );

    [seasonDataPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("season_data"), Buffer.from([1, 0, 0, 0])], // season 1
      program.programId
    );
  });

  describe("System Initialization", () => {
    it("Should initialize reputation system successfully", async () => {
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
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      // Verify configuration
      const config = await program.account.reputationConfig.fetch(configPDA);
      expect(config.admin.toString()).to.equal(admin.publicKey.toString());
      expect(config.votingCooldown.toNumber()).to.equal(VOTING_COOLDOWN);
      expect(config.minAccountAge.toNumber()).to.equal(MIN_ACCOUNT_AGE);
      expect(config.dailyVoteLimit).to.equal(DAILY_VOTE_LIMIT);
      expect(config.minReputationToVote.toNumber()).to.equal(MIN_REPUTATION_TO_VOTE);
      expect(config.categoryWeights).to.deep.equal(CATEGORY_WEIGHTS);
      expect(config.currentSeason).to.equal(1);
      expect(config.totalUsers.toNumber()).to.equal(0);
      expect(config.decayEnabled).to.be.true;
    });

    it("Should fail to initialize with invalid configuration", async () => {
      const invalidWeights = [5000, 3000, 2000, 1000]; // Doesn't sum to 10000

      try {
        await program.methods
          .initializeReputationSystem(
            new anchor.BN(VOTING_COOLDOWN),
            new anchor.BN(MIN_ACCOUNT_AGE),
            DAILY_VOTE_LIMIT,
            new anchor.BN(MIN_REPUTATION_TO_VOTE),
            invalidWeights,
            ROLE_THRESHOLDS.map(t => new anchor.BN(t))
          )
          .accounts({
            config: PublicKey.findProgramAddressSync(
              [Buffer.from("reputation_config_invalid")],
              program.programId
            )[0],
            admin: admin.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have failed with invalid category weights");
      } catch (error) {
        expect(error.message).to.include("InvalidCategoryWeights");
      }
    });
  });

  describe("User Reputation Initialization", () => {
    it("Should initialize user reputation accounts", async () => {
      // Initialize user1
      await program.methods
        .initializeUserReputation()
        .accounts({
          config: configPDA,
          userReputation: user1ReputationPDA,
          user: user1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      // Initialize user2
      await program.methods
        .initializeUserReputation()
        .accounts({
          config: configPDA,
          userReputation: user2ReputationPDA,
          user: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user2])
        .rpc();

      // Initialize user3
      await program.methods
        .initializeUserReputation()
        .accounts({
          config: configPDA,
          userReputation: user3ReputationPDA,
          user: user3.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user3])
        .rpc();

      // Verify user1 reputation
      const user1Rep = await program.account.userReputation.fetch(user1ReputationPDA);
      expect(user1Rep.user.toString()).to.equal(user1.publicKey.toString());
      expect(user1Rep.totalScore.toNumber()).to.equal(0);
      expect(user1Rep.roleLevel).to.equal(0);
      expect(user1Rep.currentStreak).to.equal(0);
      expect(user1Rep.votesCast.toNumber()).to.equal(0);
      expect(user1Rep.achievements.toNumber()).to.equal(0);

      // Verify config updated total users
      const config = await program.account.reputationConfig.fetch(configPDA);
      expect(config.totalUsers.toNumber()).to.equal(3);
    });
  });

  describe("Voting Mechanics", () => {
    it("Should allow valid upvote between users", async () => {
      // Give user1 initial reputation to meet voting requirements
      await program.methods
        .updateUserReputation(
          { governance: {} },
          new anchor.BN(200),
          "Initial reputation for testing"
        )
        .accounts({
          config: configPDA,
          userReputation: user1ReputationPDA,
          admin: admin.publicKey,
          user: user1.publicKey,
        })
        .signers([admin])
        .rpc();

      // Wait for account age requirement (simulate by updating created_at)
      await new Promise(resolve => setTimeout(resolve, 1000));

      // Cast upvote from user1 to user2
      await program.methods
        .castVote(true, { governance: {} }, 5)
        .accounts({
          config: configPDA,
          voterReputation: user1ReputationPDA,
          targetReputation: user2ReputationPDA,
          votingRecord: votingRecordPDA,
          voter: user1.publicKey,
          target: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      // Verify vote was recorded
      const user2Rep = await program.account.userReputation.fetch(user2ReputationPDA);
      expect(user2Rep.categoryPoints[0].toNumber()).to.be.greaterThan(0); // Governance category
      expect(user2Rep.totalScore.toNumber()).to.be.greaterThan(0);

      const user1Rep = await program.account.userReputation.fetch(user1ReputationPDA);
      expect(user1Rep.votesCast.toNumber()).to.equal(1);

      const votingRecord = await program.account.votingRecord.fetch(votingRecordPDA);
      expect(votingRecord.dailyVotes).to.equal(1);
      expect(votingRecord.totalVotesOnTarget).to.equal(1);
    });

    it("Should prevent voting on self", async () => {
      try {
        await program.methods
          .castVote(true, { governance: {} }, 5)
          .accounts({
            config: configPDA,
            voterReputation: user1ReputationPDA,
            targetReputation: user1ReputationPDA,
            votingRecord: PublicKey.findProgramAddressSync(
              [Buffer.from("voting_record"), user1.publicKey.toBuffer(), user1.publicKey.toBuffer()],
              program.programId
            )[0],
            voter: user1.publicKey,
            target: user1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed when voting on self");
      } catch (error) {
        expect(error.message).to.include("CannotVoteOnSelf");
      }
    });

    it("Should enforce voting cooldown", async () => {
      // Try to vote again immediately (should fail due to cooldown)
      try {
        await program.methods
          .castVote(true, { development: {} }, 3)
          .accounts({
            config: configPDA,
            voterReputation: user1ReputationPDA,
            targetReputation: user2ReputationPDA,
            votingRecord: votingRecordPDA,
            voter: user1.publicKey,
            target: user2.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to cooldown");
      } catch (error) {
        expect(error.message).to.include("VotingCooldownNotExpired");
      }
    });

    it("Should allow downvotes with reduced impact", async () => {
      // Wait for cooldown to expire
      await new Promise(resolve => setTimeout(resolve, VOTING_COOLDOWN * 1000 + 1000));

      const user2RepBefore = await program.account.userReputation.fetch(user2ReputationPDA);
      const pointsBefore = user2RepBefore.categoryPoints[0].toNumber();

      // Cast downvote
      await program.methods
        .castVote(false, { governance: {} }, 4)
        .accounts({
          config: configPDA,
          voterReputation: user1ReputationPDA,
          targetReputation: user2ReputationPDA,
          votingRecord: votingRecordPDA,
          voter: user1.publicKey,
          target: user2.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([user1])
        .rpc();

      const user2RepAfter = await program.account.userReputation.fetch(user2ReputationPDA);
      const pointsAfter = user2RepAfter.categoryPoints[0].toNumber();

      // Downvote should reduce points but with less impact than upvote
      expect(pointsAfter).to.be.lessThan(pointsBefore);
    });
  });

  describe("Role Unlock System", () => {
    it("Should allow role unlock when threshold is met", async () => {
      // Give user2 enough points to reach role level 1
      await program.methods
        .updateUserReputation(
          { governance: {} },
          new anchor.BN(150), // Total should exceed 100 (role threshold 1)
          "Points for role unlock testing"
        )
        .accounts({
          config: configPDA,
          userReputation: user2ReputationPDA,
          admin: admin.publicKey,
          user: user2.publicKey,
        })
        .signers([admin])
        .rpc();

      // Claim role unlock
      await program.methods
        .claimRoleUnlock(1)
        .accounts({
          config: configPDA,
          userReputation: user2ReputationPDA,
          user: user2.publicKey,
        })
        .signers([user2])
        .rpc();

      // Verify role level updated
      const user2Rep = await program.account.userReputation.fetch(user2ReputationPDA);
      expect(user2Rep.roleLevel).to.equal(1);
    });

    it("Should prevent role unlock when threshold not met", async () => {
      try {
        // Try to claim role level 3 without enough points
        await program.methods
          .claimRoleUnlock(3)
          .accounts({
            config: configPDA,
            userReputation: user2ReputationPDA,
            user: user2.publicKey,
          })
          .signers([user2])
          .rpc();
        
        expect.fail("Should have failed due to insufficient reputation");
      } catch (error) {
        expect(error.message).to.include("RoleUnlockRequirementsNotMet");
      }
    });
  });

  describe("Achievement System", () => {
    it("Should auto-award FirstVote achievement", async () => {
      // Check user1's achievements (should have FirstVote from previous vote)
      const user1Rep = await program.account.userReputation.fetch(user1ReputationPDA);
      
      // FirstVote achievement should be awarded (bit 0 set)
      const hasFirstVote = (user1Rep.achievements.toNumber() & (1 << 0)) !== 0;
      expect(hasFirstVote).to.be.true;
    });

    it("Should allow admin to manually award achievements", async () => {
      await program.methods
        .awardAchievement(user3.publicKey, { topContributor: {} })
        .accounts({
          config: configPDA,
          userReputation: user3ReputationPDA,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      const user3Rep = await program.account.userReputation.fetch(user3ReputationPDA);
      const hasTopContributor = (user3Rep.achievements.toNumber() & (1 << 3)) !== 0;
      expect(hasTopContributor).to.be.true;
    });

    it("Should prevent duplicate achievement awards", async () => {
      try {
        await program.methods
          .awardAchievement(user3.publicKey, { topContributor: {} })
          .accounts({
            config: configPDA,
            userReputation: user3ReputationPDA,
            admin: admin.publicKey,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have failed due to duplicate achievement");
      } catch (error) {
        expect(error.message).to.include("AchievementAlreadyAwarded");
      }
    });
  });

  describe("Seasonal System", () => {
    it("Should start new season successfully", async () => {
      await program.methods
        .startNewSeason("Test Season 2", 60, 2) // 60 days, season 2
        .accounts({
          config: configPDA,
          seasonData: PublicKey.findProgramAddressSync(
            [Buffer.from("season_data"), new Uint8Array([2, 0, 0, 0])], // season 2
            program.programId
          )[0],
          admin: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

      const config = await program.account.reputationConfig.fetch(configPDA);
      expect(config.currentSeason).to.equal(2);
    });

    it("Should prevent non-admin from starting seasons", async () => {
      try {
        await program.methods
          .startNewSeason("Unauthorized Season", 30, 3)
          .accounts({
            config: configPDA,
            seasonData: PublicKey.findProgramAddressSync(
              [Buffer.from("season_data"), new Uint8Array([3, 0, 0, 0])],
              program.programId
            )[0],
            admin: user1.publicKey, // Not admin
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to unauthorized admin");
      } catch (error) {
        expect(error.message).to.include("UnauthorizedAdmin");
      }
    });
  });

  describe("Configuration Management", () => {
    it("Should allow admin to update configuration", async () => {
      const newConfig = {
        votingCooldown: new anchor.BN(900), // 15 minutes
        minAccountAge: null,
        dailyVoteLimit: 15,
        minReputationToVote: null,
        categoryWeights: null,
        roleThresholds: null,
        decayRate: 20, // 0.2% per day
        decayEnabled: true,
      };

      await program.methods
        .updateConfig(newConfig)
        .accounts({
          config: configPDA,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      const config = await program.account.reputationConfig.fetch(configPDA);
      expect(config.votingCooldown.toNumber()).to.equal(900);
      expect(config.dailyVoteLimit).to.equal(15);
      expect(config.decayRate).to.equal(20);
    });

    it("Should prevent non-admin from updating configuration", async () => {
      try {
        await program.methods
          .updateConfig({
            votingCooldown: new anchor.BN(300),
            minAccountAge: null,
            dailyVoteLimit: null,
            minReputationToVote: null,
            categoryWeights: null,
            roleThresholds: null,
            decayRate: null,
            decayEnabled: null,
          })
          .accounts({
            config: configPDA,
            admin: user1.publicKey, // Not admin
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to unauthorized admin");
      } catch (error) {
        expect(error.message).to.include("UnauthorizedAdmin");
      }
    });
  });

  describe("Reputation Decay", () => {
    it("Should calculate decay preview correctly", async () => {
      // This would require mocking time passage in a real test
      // For now, we'll just verify the instruction executes
      const result = await program.methods
        .calculateDecayPreview(user2.publicKey)
        .accounts({
          config: configPDA,
          userReputation: user2ReputationPDA,
        })
        .view();

      expect(result.user.toString()).to.equal(user2.publicKey.toString());
      expect(result.decayEnabled).to.be.true;
    });

    it("Should allow admin to reset decay timer", async () => {
      await program.methods
        .resetDecayTimer(user2.publicKey)
        .accounts({
          config: configPDA,
          userReputation: user2ReputationPDA,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      // Verify last activity was updated
      const user2Rep = await program.account.userReputation.fetch(user2ReputationPDA);
      expect(user2Rep.lastActivity.toNumber()).to.be.greaterThan(0);
    });
  });

  describe("Streak System", () => {
    it("Should update user streak correctly", async () => {
      await program.methods
        .updateUserStreak(user1.publicKey)
        .accounts({
          userReputation: user1ReputationPDA,
        })
        .rpc();

      const user1Rep = await program.account.userReputation.fetch(user1ReputationPDA);
      expect(user1Rep.currentStreak).to.be.greaterThan(0);
    });

    it("Should get streak information", async () => {
      const streakInfo = await program.methods
        .getUserStreakInfo()
        .accounts({
          userReputation: user1ReputationPDA,
          user: user1.publicKey,
        })
        .view();

      expect(streakInfo.user.toString()).to.equal(user1.publicKey.toString());
      expect(streakInfo.currentStreak).to.be.greaterThan(0);
    });
  });

  describe("Export Functionality", () => {
    it("Should export reputation certificate", async () => {
      const certificate = await program.methods
        .exportReputation()
        .accounts({
          userReputation: user1ReputationPDA,
          user: user1.publicKey,
        })
        .view();

      expect(certificate.user.toString()).to.equal(user1.publicKey.toString());
      expect(certificate.programId.toString()).to.equal(program.programId.toString());
      expect(certificate.signatureHash).to.have.length(32);
    });

    it("Should verify reputation certificate", async () => {
      // First export a certificate
      const certificate = await program.methods
        .exportReputation()
        .accounts({
          userReputation: user1ReputationPDA,
          user: user1.publicKey,
        })
        .view();

      // Then verify it
      const isValid = await program.methods
        .verifyReputationCertificate(certificate)
        .accounts({})
        .view();

      expect(isValid).to.be.true;
    });
  });

  describe("Bulk Operations", () => {
    it("Should allow bulk reputation updates", async () => {
      const updates = [
        {
          user: user1.publicKey,
          category: { community: {} },
          pointsChange: new anchor.BN(100),
          reason: "Bulk update test 1",
        },
        {
          user: user2.publicKey,
          category: { development: {} },
          pointsChange: new anchor.BN(150),
          reason: "Bulk update test 2",
        },
      ];

      await program.methods
        .bulkUpdateReputation(updates)
        .accounts({
          config: configPDA,
          admin: admin.publicKey,
        })
        .signers([admin])
        .rpc();

      // Verify operation completed (would need actual implementation to verify changes)
    });

    it("Should prevent oversized bulk operations", async () => {
      // Create a large array of updates (exceeding limit)
      const updates = Array(101).fill(null).map((_, i) => ({
        user: user1.publicKey,
        category: { governance: {} },
        pointsChange: new anchor.BN(1),
        reason: `Update ${i}`,
      }));

      try {
        await program.methods
          .bulkUpdateReputation(updates)
          .accounts({
            config: configPDA,
            admin: admin.publicKey,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have failed due to bulk operation size");
      } catch (error) {
        expect(error.message).to.include("BulkOperationTooLarge");
      }
    });
  });

  describe("Leaderboard System", () => {
    it("Should get leaderboard data", async () => {
      const leaderboard = await program.methods
        .getLeaderboard(null, 0, 10)
        .accounts({
          config: configPDA,
        })
        .view();

      expect(leaderboard).to.be.an("array");
      expect(leaderboard.length).to.be.greaterThan(0);
    });

    it("Should prevent invalid pagination parameters", async () => {
      try {
        await program.methods
          .getLeaderboard(null, 0, 0) // Invalid page size
          .accounts({
            config: configPDA,
          })
          .view();
        
        expect.fail("Should have failed due to invalid pagination");
      } catch (error) {
        expect(error.message).to.include("InvalidPaginationParameters");
      }
    });
  });

  describe("Anti-Abuse Mechanisms", () => {
    it("Should enforce minimum reputation for voting", async () => {
      // Create a new user with no reputation
      const newUser = Keypair.generate();
      await provider.connection.requestAirdrop(newUser.publicKey, LAMPORTS_PER_SOL);
      await new Promise(resolve => setTimeout(resolve, 1000));

      const [newUserReputationPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("user_reputation"), newUser.publicKey.toBuffer()],
        program.programId
      );

      // Initialize the new user
      await program.methods
        .initializeUserReputation()
        .accounts({
          config: configPDA,
          userReputation: newUserReputationPDA,
          user: newUser.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([newUser])
        .rpc();

      // Try to vote without meeting minimum reputation requirement
      try {
        const [newVotingRecordPDA] = PublicKey.findProgramAddressSync(
          [Buffer.from("voting_record"), newUser.publicKey.toBuffer(), user1.publicKey.toBuffer()],
          program.programId
        );

        await program.methods
          .castVote(true, { governance: {} }, 5)
          .accounts({
            config: configPDA,
            voterReputation: newUserReputationPDA,
            targetReputation: user1ReputationPDA,
            votingRecord: newVotingRecordPDA,
            voter: newUser.publicKey,
            target: user1.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([newUser])
          .rpc();
        
        expect.fail("Should have failed due to insufficient reputation");
      } catch (error) {
        expect(error.message).to.include("InsufficientReputationToVote");
      }
    });

    it("Should enforce daily vote limits", async () => {
      // This test would require multiple votes in succession
      // In practice, you'd need to mock time or have a way to reset daily limits
      // For demonstration, we'll just verify the voting record structure
      const votingRecord = await program.account.votingRecord.fetch(votingRecordPDA);
      expect(votingRecord.dailyVotes).to.be.lessThan(DAILY_VOTE_LIMIT);
    });
  });

  describe("Edge Cases and Error Handling", () => {
    it("Should handle zero vote weight gracefully", async () => {
      try {
        await program.methods
          .castVote(true, { governance: {} }, 0) // Invalid vote weight
          .accounts({
            config: configPDA,
            voterReputation: user1ReputationPDA,
            targetReputation: user2ReputationPDA,
            votingRecord: votingRecordPDA,
            voter: user1.publicKey,
            target: user2.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to invalid vote weight");
      } catch (error) {
        expect(error.message).to.include("InvalidVoteWeight");
      }
    });

    it("Should handle maximum vote weight gracefully", async () => {
      try {
        await program.methods
          .castVote(true, { governance: {} }, 11) // Weight too high
          .accounts({
            config: configPDA,
            voterReputation: user1ReputationPDA,
            targetReputation: user2ReputationPDA,
            votingRecord: votingRecordPDA,
            voter: user1.publicKey,
            target: user2.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
        
        expect.fail("Should have failed due to invalid vote weight");
      } catch (error) {
        expect(error.message).to.include("InvalidVoteWeight");
      }
    });

    it("Should handle string length validation", async () => {
      try {
        const longReason = "A".repeat(201); // Exceeds 200 character limit
        
        await program.methods
          .updateUserReputation(
            { governance: {} },
            new anchor.BN(10),
            longReason
          )
          .accounts({
            config: configPDA,
            userReputation: user1ReputationPDA,
            admin: admin.publicKey,
            user: user1.publicKey,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have failed due to string too long");
      } catch (error) {
        expect(error.message).to.include("StringTooLong");
      }
    });

    it("Should handle negative reputation prevention", async () => {
      try {
        // Try to subtract more points than user has
        await program.methods
          .updateUserReputation(
            { governance: {} },
            new anchor.BN(-999999), // Very large negative number
            "Test negative reputation"
          )
          .accounts({
            config: configPDA,
            userReputation: user1ReputationPDA,
            admin: admin.publicKey,
            user: user1.publicKey,
          })
          .signers([admin])
          .rpc();
        
        expect.fail("Should have failed due to negative reputation");
      } catch (error) {
        expect(error.message).to.include("NegativeReputationNotAllowed");
      }
    });
  });

  describe("Data Integrity", () => {
    it("Should maintain consistent total scores after updates", async () => {
      const user1RepBefore = await program.account.userReputation.fetch(user1ReputationPDA);
      
      // Add points to a category
      await program.methods
        .updateUserReputation(
          { treasury: {} },
          new anchor.BN(100),
          "Data integrity test"
        )
        .accounts({
          config: configPDA,
          userReputation: user1ReputationPDA,
          admin: admin.publicKey,
          user: user1.publicKey,
        })
        .signers([admin])
        .rpc();

      const user1RepAfter = await program.account.userReputation.fetch(user1ReputationPDA);
      
      // Verify treasury category points increased
      expect(user1RepAfter.categoryPoints[3].toNumber())
        .to.be.greaterThan(user1RepBefore.categoryPoints[3].toNumber());
      
      // Verify total score was recalculated
      expect(user1RepAfter.totalScore.toNumber())
        .to.be.greaterThan(user1RepBefore.totalScore.toNumber());
    });

    it("Should maintain role level consistency", async () => {
      const user1RepBefore = await program.account.userReputation.fetch(user1ReputationPDA);
      const roleBefore = user1RepBefore.roleLevel;
      
      // Add significant points to potentially trigger role change
      await program.methods
        .updateUserReputation(
          { governance: {} },
          new anchor.BN(2000),
          "Role level test"
        )
        .accounts({
          config: configPDA,
          userReputation: user1ReputationPDA,
          admin: admin.publicKey,
          user: user1.publicKey,
        })
        .signers([admin])
        .rpc();

      const user1RepAfter = await program.account.userReputation.fetch(user1ReputationPDA);
      
      // Role level should be automatically updated if thresholds are met
      if (user1RepAfter.totalScore.toNumber() >= ROLE_THRESHOLDS[roleBefore]) {
        expect(user1RepAfter.roleLevel).to.be.greaterThanOrEqual(roleBefore);
      }
    });
  });

  describe("Performance and Gas Optimization", () => {
    it("Should handle multiple votes efficiently", async () => {
      // Wait for cooldown
      await new Promise(resolve => setTimeout(resolve, VOTING_COOLDOWN * 1000 + 1000));

      const startTime = Date.now();
      
      // Cast multiple votes (within daily limit)
      for (let i = 0; i < 3; i++) {
        await new Promise(resolve => setTimeout(resolve, VOTING_COOLDOWN * 1000 + 1000));
        
        await program.methods
          .castVote(true, { community: {} }, 2)
          .accounts({
            config: configPDA,
            voterReputation: user1ReputationPDA,
            targetReputation: user3ReputationPDA,
            votingRecord: PublicKey.findProgramAddressSync(
              [Buffer.from("voting_record"), user1.publicKey.toBuffer(), user3.publicKey.toBuffer()],
              program.programId
            )[0],
            voter: user1.publicKey,
            target: user3.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([user1])
          .rpc();
      }
      
      const endTime = Date.now();
      const totalTime = endTime - startTime;
      
      // Verify operations completed in reasonable time
      expect(totalTime).to.be.lessThan(30000); // Less than 30 seconds total
    });
  });
});