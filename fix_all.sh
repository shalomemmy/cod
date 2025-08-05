#!/bin/bash

# Comprehensive fix script for DAO Reputation Scoreboard

echo "=== Starting comprehensive fix script ==="

# Step 1: Clean previous build artifacts
echo "Cleaning previous build artifacts..."
anchor clean

# Step 2: Update dependencies
echo "Updating dependencies..."
yarn install

# Step 3: Fix any remaining issues with imports
echo "Checking for direct solana-program imports..."
grep -r "use solana_program" --include="*.rs" ./programs

# Step 4: Build with verbose output
echo "Building with verbose output..."
RUSTFLAGS="--cfg tracing_unstable" anchor build -v

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "=== All fixes applied successfully ==="
else
    echo "Build failed. Running additional diagnostics..."
    
    # Run cargo check for more detailed errors
    echo "Running cargo check..."
    cd programs/dao-reputation-scoreboard
    cargo check --verbose
    
    echo "=== Fix script completed with errors ==="
    echo "Please check the error messages above and fix any remaining issues."
fi