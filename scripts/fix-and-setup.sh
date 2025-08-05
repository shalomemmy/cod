#!/bin/bash

# DAO Reputation Scoreboard - Fix Dependencies and Setup Script
# This script fixes version conflicts and sets up the development environment

set -e

echo "🔧 Fixing DAO Reputation Scoreboard dependencies and setup..."

# Step 1: Clean existing node_modules and lock files
echo "🧹 Cleaning existing dependencies..."
rm -rf node_modules package-lock.json

# Step 2: Install updated dependencies
echo "📦 Installing updated dependencies..."
npm install

# Step 3: Check if required tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed. Please install it first."
        exit 1
    else
        echo "✅ $1 is installed"
    fi
}

echo "🔍 Checking required tools..."
check_tool "node"
check_tool "npm"
check_tool "cargo"
check_tool "solana"
check_tool "anchor"

# Step 4: Check versions
echo "📋 Version information:"
echo "Node.js: $(node --version)"
echo "npm: $(npm --version)"
echo "Rust: $(cargo --version | head -n1)"
echo "Solana: $(solana --version)"
echo "Anchor: $(anchor --version)"

# Step 5: Set Solana to devnet for development
echo "🌐 Configuring Solana for development..."
solana config set --url https://api.devnet.solana.com

# Step 6: Generate a new keypair if none exists
if [ ! -f ~/.config/solana/id.json ]; then
    echo "🔑 Generating new Solana keypair..."
    solana-keygen new --outfile ~/.config/solana/id.json --no-bip39-passphrase
fi

# Step 7: Clean Rust build cache
echo "🧹 Cleaning Rust build cache..."
cargo clean

# Step 8: Build the program
echo "🔨 Building the program..."
anchor build

# Step 9: Generate a new program keypair if needed
if [ ! -f target/deploy/dao_reputation_scoreboard-keypair.json ]; then
    echo "🔑 Generating new program keypair..."
    solana-keygen new --outfile target/deploy/dao_reputation_scoreboard-keypair.json --no-bip39-passphrase
fi

echo "✅ Setup completed successfully!"
echo ""
echo "📚 Next steps:"
echo "  1. Review the README.md for usage instructions"
echo "  2. Check examples/basic-usage.ts for implementation examples"
echo "  3. Run 'anchor test --skip-local-validator' to run tests"
echo "  4. Use './scripts/deploy.sh devnet' to deploy to devnet"
echo ""
echo "🎯 Your DAO Reputation Scoreboard is ready for development!"