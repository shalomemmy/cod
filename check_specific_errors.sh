#!/bin/bash

# Script to check for specific Rust errors

echo "=== Checking for specific Rust errors ==="

cd programs/dao-reputation-scoreboard

# Check for E0277 errors (trait bound errors)
echo "Checking for E0277 (trait bound) errors..."
cargo check 2>&1 | grep -A 3 "E0277"

# Check for E0308 errors (type mismatch)
echo "Checking for E0308 (type mismatch) errors..."
cargo check 2>&1 | grep -A 3 "E0308"

# Check for E0432 errors (unresolved import)
echo "Checking for E0432 (unresolved import) errors..."
cargo check 2>&1 | grep -A 3 "E0432"

# Check for E0599 errors (no method/field)
echo "Checking for E0599 (no method/field) errors..."
cargo check 2>&1 | grep -A 3 "E0599"

echo "=== Error check complete ==="