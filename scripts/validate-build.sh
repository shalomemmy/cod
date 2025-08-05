#!/bin/bash

# DAO Reputation Scoreboard - Build Validation Script
# This script validates the project for common build errors before attempting to build

set -e

echo "🔍 Pre-Build Validation for DAO Reputation Scoreboard..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Validation functions
validate_dependencies() {
    echo "📦 Validating dependencies..."
    
    # Check Anchor CLI
    if ! command -v anchor &> /dev/null; then
        echo -e "${RED}❌ Anchor CLI not found${NC}"
        echo "Install with: npm install -g @coral-xyz/anchor-cli"
        return 1
    fi
    
    # Check Solana CLI
    if ! command -v solana &> /dev/null; then
        echo -e "${RED}❌ Solana CLI not found${NC}"
        echo "Install from: https://docs.solana.com/cli/install-solana-cli-tools"
        return 1
    fi
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust/Cargo not found${NC}"
        echo "Install from: https://rustup.rs/"
        return 1
    fi
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        echo -e "${RED}❌ Node.js not found${NC}"
        echo "Install Node.js 16+ from: https://nodejs.org/"
        return 1
    fi
    
    echo -e "${GREEN}✅ All required dependencies found${NC}"
    return 0
}

validate_file_structure() {
    echo "📁 Validating file structure..."
    
    local required_files=(
        "Anchor.toml"
        "Cargo.toml"
        "package.json"
        "tsconfig.json"
        "programs/dao-reputation-scoreboard/Cargo.toml"
        "programs/dao-reputation-scoreboard/src/lib.rs"
        "programs/dao-reputation-scoreboard/src/state.rs"
        "programs/dao-reputation-scoreboard/src/errors.rs"
        "programs/dao-reputation-scoreboard/src/utils.rs"
        "programs/dao-reputation-scoreboard/src/instructions/mod.rs"
        "tests/dao-reputation-scoreboard.ts"
    )
    
    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            echo -e "${RED}❌ Missing required file: $file${NC}"
            return 1
        fi
    done
    
    echo -e "${GREEN}✅ All required files present${NC}"
    return 0
}

validate_rust_syntax() {
    echo "🦀 Validating Rust syntax..."
    
    cd programs/dao-reputation-scoreboard
    if cargo check --lib 2>/dev/null; then
        echo -e "${GREEN}✅ Rust syntax validation passed${NC}"
        cd ../..
        return 0
    else
        echo -e "${RED}❌ Rust syntax validation failed${NC}"
        echo "Run 'cd programs/dao-reputation-scoreboard && cargo check' for details"
        cd ../..
        return 1
    fi
}

validate_typescript_syntax() {
    echo "📝 Validating TypeScript syntax..."
    
    if npx tsc --noEmit 2>/dev/null; then
        echo -e "${GREEN}✅ TypeScript syntax validation passed${NC}"
        return 0
    else
        echo -e "${RED}❌ TypeScript syntax validation failed${NC}"
        echo "Run 'npx tsc --noEmit' for details"
        return 1
    fi
}

validate_anchor_config() {
    echo "⚓ Validating Anchor configuration..."
    
    # Check Anchor.toml
    if ! grep -q "anchor_version" Anchor.toml; then
        echo -e "${RED}❌ Anchor version not specified in Anchor.toml${NC}"
        return 1
    fi
    
    # Check program ID consistency
    local toml_program_id=$(grep -A 1 "\[programs.localnet\]" Anchor.toml | grep "dao_reputation_scoreboard" | cut -d'"' -f2)
    local lib_program_id=$(grep "declare_id!" programs/dao-reputation-scoreboard/src/lib.rs | grep -o '"[^"]*"' | tr -d '"')
    
    if [ "$toml_program_id" != "$lib_program_id" ]; then
        echo -e "${YELLOW}⚠️  Program ID mismatch between Anchor.toml and lib.rs${NC}"
        echo "Anchor.toml: $toml_program_id"
        echo "lib.rs: $lib_program_id"
        # This is a warning, not an error
    fi
    
    echo -e "${GREEN}✅ Anchor configuration validated${NC}"
    return 0
}

validate_dependencies_version() {
    echo "🔧 Validating dependency versions..."
    
    # Check Anchor version compatibility
    local anchor_version=$(grep "anchor_version" Anchor.toml | cut -d'"' -f2)
    local package_anchor=$(grep "@coral-xyz/anchor" package.json | cut -d'"' -f4 | tr -d '^~')
    
    if [ "$anchor_version" != "$package_anchor" ]; then
        echo -e "${YELLOW}⚠️  Anchor version mismatch${NC}"
        echo "Anchor.toml: $anchor_version"
        echo "package.json: $package_anchor"
    fi
    
    # Check TypeScript version
    local ts_version=$(grep '"typescript"' package.json | cut -d'"' -f4 | tr -d '^~')
    if [[ "$ts_version" < "5.0.0" ]]; then
        echo -e "${RED}❌ TypeScript version too old: $ts_version (requires 5.0.0+)${NC}"
        return 1
    fi
    
    echo -e "${GREEN}✅ Dependency versions validated${NC}"
    return 0
}

check_common_issues() {
    echo "🔍 Checking for common build issues..."
    
    # Check for missing init-if-needed feature
    if ! grep -q "init-if-needed" programs/dao-reputation-scoreboard/Cargo.toml; then
        echo -e "${RED}❌ Missing 'init-if-needed' feature in anchor-lang dependency${NC}"
        return 1
    fi
    
    # Check for missing System imports
    if ! grep -q "use anchor_lang::system_program::{System};" programs/dao-reputation-scoreboard/src/instructions/vote.rs; then
        echo -e "${RED}❌ Missing System import in vote.rs${NC}"
        return 1
    fi
    
    # Check for workspace resolver
    if ! grep -q 'resolver = "2"' Cargo.toml; then
        echo -e "${YELLOW}⚠️  Missing workspace resolver version 2${NC}"
    fi
    
    echo -e "${GREEN}✅ Common issues check passed${NC}"
    return 0
}

# Main validation sequence
main() {
    local validation_failed=false
    
    validate_dependencies || validation_failed=true
    validate_file_structure || validation_failed=true
    validate_anchor_config || validation_failed=true
    validate_dependencies_version || validation_failed=true
    check_common_issues || validation_failed=true
    validate_rust_syntax || validation_failed=true
    validate_typescript_syntax || validation_failed=true
    
    if [ "$validation_failed" = true ]; then
        echo ""
        echo -e "${RED}❌ Validation failed! Please fix the issues above before building.${NC}"
        exit 1
    else
        echo ""
        echo -e "${GREEN}🎉 All validations passed! Your project is ready to build.${NC}"
        echo ""
        echo "📚 Next steps:"
        echo "  1. Run: ./scripts/build-and-test.sh"
        echo "  2. Or manually: anchor build"
        echo ""
    fi
}

# Run main function
main