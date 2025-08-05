#!/bin/bash

# DAO Reputation Scoreboard Deployment Script
# Usage: ./scripts/deploy.sh [devnet|mainnet]

set -e

NETWORK=${1:-devnet}

echo "🚀 Deploying DAO Reputation Scoreboard to $NETWORK..."

# Validate network parameter
if [[ "$NETWORK" != "devnet" && "$NETWORK" != "mainnet" ]]; then
    echo "❌ Invalid network. Use 'devnet' or 'mainnet'"
    exit 1
fi

# Set appropriate RPC URL
if [[ "$NETWORK" == "devnet" ]]; then
    RPC_URL="https://api.devnet.solana.com"
    echo "🌐 Using Devnet RPC: $RPC_URL"
elif [[ "$NETWORK" == "mainnet" ]]; then
    RPC_URL="https://api.mainnet-beta.solana.com"
    echo "🌐 Using Mainnet RPC: $RPC_URL"
    
    # Additional confirmation for mainnet
    echo "⚠️  You are about to deploy to MAINNET. This will cost real SOL."
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Deployment cancelled"
        exit 1
    fi
fi

# Configure Solana CLI
echo "⚙️  Configuring Solana CLI..."
solana config set --url $RPC_URL

# Check wallet balance
BALANCE=$(solana balance --lamports)
MIN_BALANCE=2000000000  # 2 SOL in lamports

if [[ "$BALANCE" -lt "$MIN_BALANCE" ]]; then
    echo "❌ Insufficient balance. You need at least 2 SOL for deployment."
    echo "💰 Current balance: $(echo "scale=9; $BALANCE / 1000000000" | bc) SOL"
    
    if [[ "$NETWORK" == "devnet" ]]; then
        echo "🎁 Requesting airdrop for devnet..."
        solana airdrop 2
    else
        echo "💸 Please fund your wallet for mainnet deployment"
        exit 1
    fi
fi

# Build the program
echo "🔨 Building program..."
anchor build

# Deploy the program
echo "📡 Deploying to $NETWORK..."
if [[ "$NETWORK" == "devnet" ]]; then
    anchor deploy --provider.cluster devnet
elif [[ "$NETWORK" == "mainnet" ]]; then
    anchor deploy --provider.cluster mainnet-beta
fi

# Get the program ID
PROGRAM_ID=$(solana address -k target/deploy/dao_reputation_scoreboard-keypair.json)
echo "✅ Program deployed successfully!"
echo "📍 Program ID: $PROGRAM_ID"

# Update Anchor.toml with the new program ID
echo "📝 Updating Anchor.toml..."
if [[ "$NETWORK" == "devnet" ]]; then
    sed -i.bak "s/dao_reputation_scoreboard = \".*\"/dao_reputation_scoreboard = \"$PROGRAM_ID\"/" Anchor.toml
fi

# Verify deployment
echo "🔍 Verifying deployment..."
solana program show $PROGRAM_ID

echo "\n✅ Deployment completed successfully!"
echo "\n📋 Deployment Summary:"
echo "  Network: $NETWORK"
echo "  Program ID: $PROGRAM_ID"
echo "  RPC URL: $RPC_URL"

echo "\n📚 Next steps:"
echo "  1. Initialize the reputation system using the initialize script"
echo "  2. Test the deployment with example scripts"
echo "  3. Configure your frontend/backend with the new program ID"

if [[ "$NETWORK" == "mainnet" ]]; then
    echo "\n⚠️  MAINNET DEPLOYMENT CHECKLIST:"
    echo "  ✓ Program deployed"
    echo "  ☐ Initialize reputation system"
    echo "  ☐ Set up admin operations"
    echo "  ☐ Configure monitoring"
    echo "  ☐ Set up backup procedures"
    echo "  ☐ Test all functionality"
fi