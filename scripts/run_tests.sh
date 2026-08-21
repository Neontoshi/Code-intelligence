#!/bin/bash
set -euo pipefail

echo "🧪 Code Intelligence Test Suite v2"
echo "=================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

FAILED=0
TOTAL_TESTS=0
PASSED_TESTS=0

run_test() {
    local name=$1
    local cmd=$2
    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo -n "  $name... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAILED${NC}"
        FAILED=1
    fi
}

run_test_verbose() {
    local name=$1
    local cmd=$2
    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    echo -e "\n  ${BLUE}$name${NC}"
    if eval "$cmd"; then
        echo -e "  ${GREEN}✅ PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "  ${RED}❌ FAILED${NC}"
        FAILED=1
    fi
}

echo "📦 Checking binaries..."
run_test "ci" "cargo run --quiet --bin ci -- --help"
run_test "dead_code_check" "cargo run --quiet --bin dead_code_check -- --help"
run_test "dead_code_dashboard" "cargo run --quiet --bin dead_code_dashboard -- --help"
run_test "dedup_check" "cargo run --quiet --bin dedup_check -- --help"

echo ""
echo "📦 Running tests..."
run_test_verbose "Unit tests" "cargo test --test unit"
run_test_verbose "Integration tests" "cargo test --test integration"
run_test_verbose "Property tests" "cargo test --test property_tests"
run_test_verbose "Fuzz tests" "cargo test --test fuzz_tests"

echo ""
echo "📦 Running adversarial tests..."
run_test_verbose "Adversarial fixtures" "cargo test --test adversarial_tests -- --nocapture"

echo ""
echo "📦 Testing self-analysis..."
if cargo run --quiet --bin dead_code_check . --threshold 0.90 --max-files 100 > /tmp/self_analysis.log 2>&1; then
    echo -e "  ${GREEN}✅ Self-analysis PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "  ${RED}❌ Self-analysis FAILED${NC}"
    FAILED=1
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

echo ""
echo "📊 Test Summary"
echo "=================================="
echo "  Total tests:  $TOTAL_TESTS"
echo "  Passed:       $PASSED_TESTS"
echo "  Failed:       $((TOTAL_TESTS - PASSED_TESTS))"

if [ $FAILED -eq 0 ]; then
    echo -e "\n${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "\n${RED}❌ Some tests failed!${NC}"
    exit 1
fi
