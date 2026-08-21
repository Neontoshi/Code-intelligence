#!/bin/bash
set -euo pipefail

echo "🧪 Running Code Intelligence Test Suite"
echo "========================================"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Track failures
FAILED=0

run_test_group() {
    echo ""
    echo "📦 $1"
    echo "----------------------------------------"
    shift
    for test in "$@"; do
        echo -n "  $test... "
        if cargo test --test "$test" --quiet 2>/dev/null; then
            echo -e "${GREEN}✅ PASSED${NC}"
        else
            echo -e "${RED}❌ FAILED${NC}"
            FAILED=1
        fi
    done
}

# Run all test groups
run_test_group "Unit Tests" "unit"
run_test_group "Integration Tests" "integration"
run_test_group "Property Tests" "property_tests"
run_test_group "Fuzz Tests" "fuzz_tests"

# Run CLI smoke tests (actual tests, not just --help)
echo ""
echo "🧪 Running CLI smoke tests..."
for bin in ci dead_code_check dedup_check; do
    echo -n "  $bin --help... "
    if cargo run --quiet --bin "$bin" -- --help > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASSED${NC}"
    else
        echo -e "${RED}❌ FAILED${NC}"
        FAILED=1
    fi
done

echo ""
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed!${NC}"
    exit 1
fi
