#!/bin/bash

# DAO Reputation Scoreboard - Build and Test Script
# This script builds the Anchor project and runs tests

set -e

echo "🚀 Building and Testing DAO Reputation Scoreboard..."

# Step 1: Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf target/
rm -rf node_modules/.cache/

# Step 2: Install TypeScript dependencies
echo "📦 Installing/updating dependencies..."
npm install

# Step 3: Check Anchor version compatibility
echo "🔍 Checking Anchor version..."
ANCHOR_VERSION=$(anchor --version | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+')
echo "Anchor version: $ANCHOR_VERSION"

if [[ "$ANCHOR_VERSION" != "0.31.1" ]]; then
    echo "⚠️  Warning: Expected Anchor 0.31.1, found $ANCHOR_VERSION"
    echo "   This may cause compatibility issues."
fi

# Step 4: Build the Rust program
echo "🔨 Building Rust program..."
cd programs/dao-reputation-scoreboard
cargo check --lib
cargo build --release
cd ../..

# Step 5: Build with Anchor
echo "🏗️  Building with Anchor..."
anchor build

# Step 6: Verify the build was successful
if [ -f "target/deploy/dao_reputation_scoreboard.so" ]; then
    echo "✅ Build successful!"
    echo "📁 Program binary: target/deploy/dao_reputation_scoreboard.so"
    echo "📋 IDL file: target/idl/dao_reputation_scoreboard.json"
    echo "📝 TypeScript types: target/types/dao_reputation_scoreboard.ts"
else
    echo "❌ Build failed - no program binary found"
    exit 1
fi

# Step 7: Run TypeScript compilation check
echo "🔍 Checking TypeScript compilation..."
if npx tsc --noEmit; then
    echo "✅ TypeScript compilation successful"
else
    echo "❌ TypeScript compilation failed"
    echo "Please fix TypeScript errors before proceeding"
    exit 1
fi

# Step 8: Check if IDL and types were generated
echo "🔍 Verifying build artifacts..."
if [ ! -f "target/idl/dao_reputation_scoreboard.json" ]; then
    echo "❌ IDL file not generated"
    exit 1
fi

if [ ! -f "target/types/dao_reputation_scoreboard.ts" ]; then
    echo "❌ TypeScript types not generated"
    exit 1
fi

# Step 9: Run tests (skip if no local validator)
echo "🧪 Running tests..."
if command -v solana-test-validator &> /dev/null; then
    echo "Running full test suite with local validator..."
    # Use timeout to prevent hanging
    timeout 300 anchor test || {
        echo "❌ Tests failed or timed out after 5 minutes"
        echo "Check test logs for more details"
        exit 1
    }
else
    echo "⚠️  Local validator not found, skipping integration tests"
    echo "   To run full tests, install solana-test-validator"
    echo "   For now, running syntax validation only..."
    if npx ts-node --transpile-only tests/dao-reputation-scoreboard.ts 2>/dev/null; then
        echo "✓ Test syntax validation passed"
    else
        echo "❌ Test syntax validation failed"
        echo "Check your test file for syntax errors"
        exit 1
    fi
fi

echo ""
echo "🎉 Build and test completed successfully!"
echo ""
echo "📚 Next steps:"
echo "  1. Deploy to devnet: ./scripts/deploy.sh devnet"
echo "  2. Initialize system: npx ts-node scripts/initialize-system.ts"
echo "  3. Run example: npx ts-node examples/basic-usage.ts"
echo ""
echo "🎯 Your DAO Reputation Scoreboard is ready!"