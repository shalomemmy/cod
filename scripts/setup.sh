#!/bin/bash

# DAO Reputation Scoreboard Setup Script
# This script sets up the development environment

set -e

echo "🚀 Setting up DAO Reputation Scoreboard development environment..."

# Check if required tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed. Please install it first."
        exit 1
    else
        echo "✅ $1 is installed"
    fi
}

echo "\n🔍 Checking required tools..."
check_tool "node"
check_tool "npm"
check_tool "cargo"
check_tool "solana"
check_tool "anchor"

# Check versions
echo "\n📋 Version information:"
echo "Node.js: $(node --version)"
echo "npm: $(npm --version)"
echo "Rust: $(cargo --version | head -n1)"
echo "Solana: $(solana --version)"
echo "Anchor: $(anchor --version)"

# Install Node.js dependencies
echo "\n📦 Installing Node.js dependencies..."
npm install

# Create necessary directories
echo "\n📁 Creating project directories..."
mkdir -p scripts
mkdir -p examples
mkdir -p docs

# Set Solana to devnet for development
echo "\n🌐 Configuring Solana for development..."
solana config set --url https://api.devnet.solana.com

# Generate a new keypair if none exists
if [ ! -f ~/.config/solana/id.json ]; then
    echo "🔑 Generating new Solana keypair..."
    solana-keygen new --outfile ~/.config/solana/id.json --no-bip39-passphrase
fi

# Build the program
echo "\n🔨 Building the program..."
anchor build

# Run tests to verify everything works
echo "\n🧪 Running tests..."
anchor test --skip-local-validator

echo "\n✅ Setup completed successfully!"
echo "\n📚 Next steps:"
echo "  1. Review the README.md for usage instructions"
echo "  2. Check examples/basic-usage.ts for implementation examples"
echo "  3. Run 'anchor test' to run the full test suite"
echo "  4. Use 'npm run deploy:devnet' to deploy to devnet"
echo "\n🎯 Happy coding!"