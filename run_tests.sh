#!/bin/bash
set -e

echo "🧪 Code Intelligence - Phase 1 Test Suite"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Function to run a test
run_test() {
    echo -n "  Testing $1... "
    if cargo run --quiet --bin "$1" -- --help > /dev/null 2>&1; then
        echo -e "${GREEN}✅ OK${NC}"
        return 0
    else
        echo -e "${RED}❌ FAILED${NC}"
        return 1
    fi
}

# Function to run a test with output
run_test_verbose() {
    echo ""
    echo "📊 Testing $1..."
    cargo run --quiet --bin "$1" -- "$2" 2>&1 | head -30 || true
    echo ""
}

echo "📦 Checking binaries..."
run_test verify_ground_truth
run_test split_repositories
run_test temporal_evaluation
run_test hard_negative_dataset
run_test evaluate_metrics
run_test evaluate_per_language_detailed
run_test calibration_analysis

echo ""
echo "📊 Running verification on code-intelligence itself..."
cargo run --bin verify_ground_truth -- . --count 5 --output test_verification.json 2>&1 | head -40 || true

echo ""
echo "📊 Running hard-negative detection..."
cargo run --bin hard_negative_dataset -- . --count 5 --min-confidence 0.5 --output test_hard_negatives.json 2>&1 | head -40 || true

echo ""
echo "📊 Running adversarial tests..."
cargo test --test adversarial_tests -- --nocapture 2>&1 | head -60 || true

echo ""
echo -e "${GREEN}✅ Test suite complete!${NC}"
