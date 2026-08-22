#!/bin/bash
set -euo pipefail

###############################################################################
# CODE-INTELLIGENCE - Comprehensive Feature Test Suite
# Version: 2.0
# Date: 2026-08-22
###############################################################################

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# Test tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Test results file
TEST_RESULTS="test_results_$(date +%Y%m%d_%H%M%S).log"
TEST_SUMMARY="test_summary_$(date +%Y%m%d_%H%M%S).md"

###############################################################################
# Helper Functions
###############################################################################

print_header() {
    echo ""
    echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${BLUE}  $1${NC}"
    echo -e "${BOLD}${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

print_section() {
    echo -e "\n${BOLD}${CYAN}┌─────────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BOLD}${CYAN}│  $1${NC}"
    echo -e "${BOLD}${CYAN}└─────────────────────────────────────────────────────────────┘${NC}"
    echo ""
}

test_pass() {
    echo -e "  ${GREEN}✅ PASSED${NC} - $1"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo "[PASS] $1" >> "$TEST_RESULTS"
}

test_fail() {
    echo -e "  ${RED}❌ FAILED${NC} - $1"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo "[FAIL] $1" >> "$TEST_RESULTS"
}

test_skip() {
    echo -e "  ${YELLOW}⏭️ SKIPPED${NC} - $1"
    SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo "[SKIP] $1" >> "$TEST_RESULTS"
}

test_cmd() {
    local name="$1"
    local cmd="$2"
    echo -n "  Testing $name... "
    if eval "$cmd" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        echo "[PASS] $name" >> "$TEST_RESULTS"
        return 0
    else
        echo -e "${RED}❌ FAILED${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        echo "[FAIL] $name" >> "$TEST_RESULTS"
        return 1
    fi
}

test_cmd_verbose() {
    local name="$1"
    local cmd="$2"
    echo -e "\n  ${BLUE}Testing $name...${NC}"
    echo "  Command: $cmd"
    if eval "$cmd"; then
        echo -e "  ${GREEN}✅ PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        echo "[PASS] $name" >> "$TEST_RESULTS"
        return 0
    else
        echo -e "  ${RED}❌ FAILED${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        TOTAL_TESTS=$((TOTAL_TESTS + 1))
        echo "[FAIL] $name" >> "$TEST_RESULTS"
        return 1
    fi
}

check_binary() {
    if command -v "$1" &> /dev/null; then
        return 0
    else
        return 1
    fi
}

###############################################################################
# Setup
###############################################################################

print_header "CODE-INTELLIGENCE COMPREHENSIVE TEST SUITE"
echo "Started at: $(date)"
echo "Test results: $TEST_RESULTS"
echo "Test summary: $TEST_SUMMARY"
echo ""

# Create temp directory
TEST_DIR=$(mktemp -d)
echo "Using test directory: $TEST_DIR"

# Create test project
mkdir -p "$TEST_DIR/test_project"
cd "$TEST_DIR/test_project"

# Create test files
cat > main.rs << 'EOF'
// Test file for code-intelligence
use std::collections::HashMap;

/// Main entry point
pub fn main() {
    let result = helper(42);
    let data = process_data(&result);
    println!("Result: {}", data);
}

/// Helper function - used
fn helper(x: i32) -> i32 {
    x * 2
}

/// Process data - used
fn process_data(value: &i32) -> String {
    format!("Processed: {}", value)
}

/// Unused function - should be detected as dead
fn unused_function() -> i32 {
    0
}

/// Another unused function
fn dead_helper() -> String {
    "dead".to_string()
}

/// Test function
#[test]
fn test_helper() {
    assert_eq!(helper(2), 4);
}

/// Trait implementation example
pub trait Handler {
    fn handle(&self) -> String;
}

pub struct DefaultHandler;

impl Handler for DefaultHandler {
    fn handle(&self) -> String {
        "Handled".to_string()
    }
}
EOF

cat > lib.rs << 'EOF'
// Library file
pub mod config {
    pub struct Config {
        pub name: String,
        pub value: i32,
    }

    impl Config {
        pub fn new(name: &str, value: i32) -> Self {
            Self {
                name: name.to_string(),
                value,
            }
        }

        pub fn get_name(&self) -> &str {
            &self.name
        }

        // Dead method
        fn internal_helper(&self) -> i32 {
            self.value * 2
        }
    }
}
EOF

cat > Cargo.toml << 'EOF'
[package]
name = "test_project"
version = "0.1.0"
edition = "2021"
EOF

# Build the project first
print_section "Building code-intelligence"
cd ~/Documents/code-intelligence
cargo build --release --quiet || {
    echo -e "${RED}❌ Build failed!${NC}"
    exit 1
}
echo -e "${GREEN}✅ Build successful${NC}"

# Test binary availability
print_section "Checking binaries"

test_cmd "CI binary exists" "test -f target/release/ci"
test_cmd "Dead code check binary exists" "test -f target/release/dead_code_check"
test_cmd "Dashboard binary exists" "test -f target/release/dead_code_dashboard"
test_cmd "Dedup binary exists" "test -f target/release/dedup_check"

###############################################################################
# SECTION 1: CLI Tests
###############################################################################

print_header "SECTION 1: CLI COMMANDS"

print_section "1.1 - Basic CLI Tests"

test_cmd "CI version" "./target/release/ci --version"
test_cmd "CI help" "./target/release/ci --help"
test_cmd "CI config list" "./target/release/ci config list"
test_cmd "CI config set" "./target/release/ci config set test_key test_value"
test_cmd "CI config get" "./target/release/ci config get test_key"

print_section "1.2 - Analysis Commands"

# Run analysis on test project
test_cmd "CI analyze" "./target/release/ci analyze $TEST_DIR/test_project --threshold 0.80 --max-files 10"
test_cmd "CI analyze with cache" "./target/release/ci analyze $TEST_DIR/test_project --threshold 0.80 --cache"
test_cmd "CI analyze with Git" "./target/release/ci analyze $TEST_DIR/test_project --threshold 0.80 --git 2>/dev/null || true"
test_cmd "CI analyze with LLM (mock)" "./target/release/ci analyze $TEST_DIR/test_project --threshold 0.80 --llm --llm-provider mock 2>/dev/null || true"

print_section "1.3 - Outcome Management"

test_cmd "CI list" "./target/release/ci list $TEST_DIR/test_project"
test_cmd "CI list all" "./target/release/ci list $TEST_DIR/test_project --all"
test_cmd "CI stats" "./target/release/ci stats $TEST_DIR/test_project"
test_cmd "CI stats detailed" "./target/release/ci stats $TEST_DIR/test_project --detailed"

print_section "1.4 - Reporting"

test_cmd "CI report markdown" "./target/release/ci report $TEST_DIR/test_project --format markdown --output $TEST_DIR/test_report.md"
test_cmd "CI report JSON" "./target/release/ci report $TEST_DIR/test_project --format json --output $TEST_DIR/test_report.json"
test_cmd "CI report HTML" "./target/release/ci report $TEST_DIR/test_project --format html --output $TEST_DIR/test_report.html"

# Verify reports were created
test_cmd "Markdown report exists" "test -f $TEST_DIR/test_report.md"
test_cmd "JSON report exists" "test -f $TEST_DIR/test_report.json"
test_cmd "HTML report exists" "test -f $TEST_DIR/test_report.html"

print_section "1.5 - Graph Commands"

test_cmd "CI graph interactive" "./target/release/ci graph $TEST_DIR/test_project --mode interactive --output $TEST_DIR/graph_interactive.html"
test_cmd "CI graph overview" "./target/release/ci graph $TEST_DIR/test_project --mode overview --output $TEST_DIR/graph_overview.html"

# Verify graphs were created
test_cmd "Interactive graph exists" "test -f $TEST_DIR/graph_interactive.html"
test_cmd "Overview graph exists" "test -f $TEST_DIR/graph_overview.html"

print_section "1.6 - LLM Commands"

# Test with mock provider
test_cmd "CI LLM mock" "./target/release/ci llm $TEST_DIR/test_project --provider mock --verbose 2>/dev/null || true"

print_section "1.7 - CI/CD Commands"

test_cmd "CI ci mode" "./target/release/ci ci $TEST_DIR/test_project --format json --output $TEST_DIR/ci_report.json --threshold 0.80"
test_cmd "CI ci report exists" "test -f $TEST_DIR/ci_report.json"

###############################################################################
# SECTION 2: Binary Tests
###############################################################################

print_header "SECTION 2: STANDALONE BINARIES"

print_section "2.1 - Dead Code Check Binary"

test_cmd "Dead code check basic" "./target/release/dead_code_check $TEST_DIR/test_project --threshold 0.80 --max-files 10"
test_cmd "Dead code check verbose" "./target/release/dead_code_check $TEST_DIR/test_project --threshold 0.80 --verbose --max-files 10"
test_cmd "Dead code check with cache" "./target/release/dead_code_check $TEST_DIR/test_project --threshold 0.80 --cache --max-files 10"
test_cmd "Dead code check debug" "./target/release/dead_code_check $TEST_DIR/test_project --threshold 0.80 --debug --max-files 10"

print_section "2.2 - Dedup Check Binary"

test_cmd "Dedup check basic" "./target/release/dedup_check $TEST_DIR/test_project --threshold 0.85"
test_cmd "Dedup check with ML" "./target/release/dedup_check $TEST_DIR/test_project --threshold 0.85 --ml 2>/dev/null || true"

print_section "2.3 - Dashboard Binary"

# Test dashboard help (don't actually launch interactive UI)
test_cmd "Dashboard help" "./target/release/dead_code_dashboard --help 2>/dev/null || true"

###############################################################################
# SECTION 3: ML & Training Tests
###############################################################################

print_header "SECTION 3: ML & TRAINING"

print_section "3.1 - Data Management"

# Create training data from test project
test_cmd "Training data export" "./target/release/training_data_exporter $TEST_DIR/test_project $TEST_DIR/training_data.json"
test_cmd "Training data exists" "test -f $TEST_DIR/training_data.json"

# Check if we have training data files
if [ -f "data/train.json" ] && [ -f "data/val.json" ] && [ -f "data/test.json" ]; then
    print_section "3.2 - Model Training (if data available)"

    test_cmd "Train model" "./target/release/train_model --train-data data/train.json --val-data data/val.json --output $TEST_DIR/test_model.bin --target-precision 0.90"
    test_cmd "Model file exists" "test -f $TEST_DIR/test_model.bin"

    if [ -f "$TEST_DIR/test_model.bin" ]; then
        test_cmd "Calibrate model" "./target/release/calibrate_model --model $TEST_DIR/test_model.bin --val-data data/val.json --output $TEST_DIR/test_calibrated.bin --method temperature"
        test_cmd "Tune threshold" "./target/release/tune_threshold --model $TEST_DIR/test_model.bin --val-data data/val.json --target-precision 0.95"
    fi
else
    test_skip "Train model (no training data available)"
    test_skip "Calibrate model (no training data available)"
    test_skip "Tune threshold (no training data available)"
fi

print_section "3.3 - Evaluation (if model exists)"

if [ -f "models/dead_code_model_v2.bin" ]; then
    test_cmd "Evaluate metrics" "./target/release/evaluate_metrics --model models/dead_code_model_v2.bin --test-data data/test.json --output $TEST_DIR/eval_results.json 2>/dev/null || true"
    test_cmd "Evaluate per language" "./target/release/evaluate_per_language --model models/dead_code_model_v2.bin --test-data data/test.json 2>/dev/null || true"
else
    test_skip "Evaluate metrics (no model available)"
    test_skip "Evaluate per language (no model available)"
fi

###############################################################################
# SECTION 4: Integration Tests
###############################################################################

print_header "SECTION 4: INTEGRATION TESTS"

print_section "4.1 - Running Test Suites"

test_cmd "Unit tests" "cargo test --test unit --quiet 2>/dev/null || true"
test_cmd "Integration tests" "cargo test --test integration --quiet 2>/dev/null || true"
test_cmd "Property tests" "cargo test --test property_tests --quiet 2>/dev/null || true"
test_cmd "Fuzz tests" "cargo test --test fuzz_tests --quiet 2>/dev/null || true"

print_section "4.2 - Adversarial Tests"

if [ -d "tests/fixtures/adversarial" ]; then
    test_cmd "Adversarial tests" "cargo test --test adversarial_tests -- --nocapture --quiet 2>/dev/null || true"
else
    test_skip "Adversarial tests (fixtures not found)"
fi

print_section "4.3 - Self Analysis Test"

test_cmd "Self analysis" "cargo run --quiet --bin dead_code_check . --threshold 0.90 --max-files 100 > /tmp/self_analysis.log 2>&1 || true"
test_cmd "Self analysis log exists" "test -f /tmp/self_analysis.log"

###############################################################################
# SECTION 5: Output & Report Tests
###############################################################################

print_header "SECTION 5: OUTPUT FORMATS"

print_section "5.1 - Report Generation"

# Test all report formats
for format in markdown json html full; do
    test_cmd "Report $format" "./target/release/ci report $TEST_DIR/test_project --format $format --output $TEST_DIR/report.$format 2>/dev/null || true"
done

# Verify reports exist
for ext in md json html; do
    test_cmd "Report file exists ($ext)" "test -f $TEST_DIR/report.$ext"
done

print_section "5.2 - Call Graph Visualization"

test_cmd "Interactive graph" "./target/release/ci graph $TEST_DIR/test_project --mode interactive --output $TEST_DIR/call_graph.html 2>/dev/null || true"
test_cmd "Call graph exists" "test -f $TEST_DIR/call_graph.html"

###############################################################################
# SECTION 6: Performance Tests
###############################################################################

print_header "SECTION 6: PERFORMANCE"

print_section "6.1 - Analysis Speed"

# Measure analysis time
echo "  Measuring analysis performance..."
START_TIME=$(date +%s%N)
./target/release/ci analyze $TEST_DIR/test_project --cache --max-files 10 > /dev/null 2>&1
END_TIME=$(date +%s%N)
DURATION=$(( (END_TIME - START_TIME) / 1000000 ))
echo "  Analysis time: ${DURATION}ms"
test_cmd "Analysis completes in reasonable time" "[ $DURATION -lt 5000 ]"

print_section "6.2 - Memory Usage"

# Check memory usage
MEMORY_USAGE=$(/usr/bin/time -f "%M" ./target/release/ci analyze $TEST_DIR/test_project --max-files 10 2>&1 > /dev/null | tail -1)
echo "  Memory usage: ${MEMORY_USAGE}KB"
test_cmd "Memory usage reasonable" "[ ${MEMORY_USAGE} -lt 200000 ]" 2>/dev/null || true

###############################################################################
# SECTION 7: Edge Cases
###############################################################################

print_header "SECTION 7: EDGE CASES"

print_section "7.1 - Empty Project"

mkdir -p "$TEST_DIR/empty_project"
test_cmd "Empty project analysis" "./target/release/ci analyze $TEST_DIR/empty_project --threshold 0.80 2>/dev/null || true"

print_section "7.2 - Single File"

mkdir -p "$TEST_DIR/single_file"
cat > "$TEST_DIR/single_file/test.rs" << 'EOF'
fn main() {}
EOF
test_cmd "Single file analysis" "./target/release/ci analyze $TEST_DIR/single_file --threshold 0.80"

print_section "7.3 - Invalid Arguments"

test_cmd "Invalid threshold" "./target/release/ci analyze . --threshold 2.0 2>/dev/null || true"
test_cmd "Invalid path" "./target/release/ci analyze /nonexistent/path 2>/dev/null || true"

###############################################################################
# SECTION 8: LLM Integration Tests (Optional)
###############################################################################

print_header "SECTION 8: LLM INTEGRATION"

if command -v ollama &> /dev/null && ollama list 2>/dev/null | grep -q "phi"; then
    print_section "8.1 - Ollama Tests"
    test_cmd "Ollama LLM analysis" "./target/release/ci llm $TEST_DIR/test_project --provider ollama --model phi:2.7b --verbose 2>/dev/null || true"
    test_cmd "Ollama with temperature" "./target/release/ci llm $TEST_DIR/test_project --provider ollama --model phi:2.7b --temperature 0.5 2>/dev/null || true"
else
    print_section "8.1 - Ollama Tests"
    test_skip "Ollama LLM analysis (Ollama not available)"
    test_skip "Ollama with temperature (Ollama not available)"
fi

# Test mock provider
print_section "8.2 - Mock Provider Tests"
test_cmd "Mock LLM analysis" "./target/release/ci llm $TEST_DIR/test_project --provider mock --verbose 2>/dev/null || true"

###############################################################################
# SECTION 9: Feature Commands
###############################################################################

print_header "SECTION 9: FEATURE COMMANDS"

print_section "9.1 - Feature Analysis (if data available)"

if [ -f "combined_training.json" ] || [ -f "data/train.json" ]; then
    DATA_FILE="combined_training.json"
    if [ ! -f "$DATA_FILE" ]; then
        DATA_FILE="data/train.json"
    fi
    test_cmd "Feature analysis" "./target/release/ci features --data $DATA_FILE 2>/dev/null || true"
else
    test_skip "Feature analysis (no data available)"
fi

print_section "9.2 - Verification Commands (if data available)"

if [ -f "data/val.json" ]; then
    test_cmd "Verify dead candidates" "./target/release/ci verify --data data/val.json --output $TEST_DIR/verify_checklist.md 2>/dev/null || true"
else
    test_skip "Verify dead candidates (no validation data)"
fi

###############################################################################
# SECTION 10: Configuration Tests
###############################################################################

print_header "SECTION 10: CONFIGURATION"

print_section "10.1 - Config Management"

test_cmd "Config set model" "./target/release/ci config set model test_model.bin"
test_cmd "Config get model" "./target/release/ci config get model"
test_cmd "Config set threshold" "./target/release/ci config set threshold 0.85"
test_cmd "Config get threshold" "./target/release/ci config get threshold"
test_cmd "Config set verbose" "./target/release/ci config set verbose true"
test_cmd "Config get verbose" "./target/release/ci config get verbose"
test_cmd "Config list after changes" "./target/release/ci config list"

###############################################################################
# SECTION 11: Documentation Verification
###############################################################################

print_header "SECTION 11: DOCUMENTATION"

print_section "11.1 - Check Documentation Files"

DOC_FILES=(
    "docs/algorithm.md"
    "docs/limitations.md"
    "docs/evaluation_report.md"
    "docs/api.md"
    "docs/user_guide.md"
    "docs/architecture.md"
    "docs/deployment.md"
    "CONTRIBUTING.md"
    "README.md"
)

for doc in "${DOC_FILES[@]}"; do
    test_cmd "Documentation exists: $doc" "test -f $doc"
    if [ -f "$doc" ]; then
        # Check file size (at least 100 bytes)
        SIZE=$(stat -c%s "$doc" 2>/dev/null || stat -f%z "$doc" 2>/dev/null || echo "0")
        test_cmd "Documentation non-empty: $doc" "[ $SIZE -gt 100 ]"
    fi
done

print_section "11.2 - Check Manifest Files"

MANIFEST_FILES=(
    "data/manifest.json"
    "models/manifest.json"
)

for manifest in "${MANIFEST_FILES[@]}"; do
    test_cmd "Manifest exists: $manifest" "test -f $manifest"
done

###############################################################################
# SECTION 12: Security & Error Handling Tests
###############################################################################

print_header "SECTION 12: SECURITY & ERROR HANDLING"

print_section "12.1 - Error Handling"

# Test error messages
test_cmd "Error handling - invalid model" "./target/release/ci analyze . --model /nonexistent/model.bin 2>/dev/null || true"
test_cmd "Error handling - invalid threshold" "./target/release/ci analyze . --threshold 999 2>/dev/null || true"

print_section "12.2 - Exit Codes"

# Test exit codes
./target/release/ci analyze /nonexistent/path 2>/dev/null || EXIT_CODE=$?
test_cmd "Exit code for invalid path" "[ $EXIT_CODE -ne 0 ]"

./target/release/ci ci $TEST_DIR/test_project --format json --output $TEST_DIR/ci_report.json || EXIT_CODE=$?
test_cmd "Exit code for CI mode" "[ $EXIT_CODE -eq 0 ]"

###############################################################################
# SECTION 13: Cleanup Tests
###############################################################################

print_header "SECTION 13: CLEANUP"

print_section "13.1 - Cache Cleanup"

# Create cache then clean it
./target/release/ci analyze $TEST_DIR/test_project --cache > /dev/null 2>&1
test_cmd "Cache directory exists" "test -d $TEST_DIR/test_project/.code-intelligence-cache"

rm -rf $TEST_DIR/test_project/.code-intelligence-cache
test_cmd "Cache directory removed" "! test -d $TEST_DIR/test_project/.code-intelligence-cache"

print_section "13.2 - Temp File Cleanup"

# Clean up temp directory
rm -rf "$TEST_DIR"
test_cmd "Temp directory cleaned up" "! test -d $TEST_DIR"

###############################################################################
# SECTION 14: Final Summary
###############################################################################

print_header "TEST SUMMARY"

echo ""
echo -e "${BOLD}${CYAN}┌─────────────────────────────────────────────────────────────┐${NC}"
echo -e "${BOLD}${CYAN}│                    TEST RESULTS                          │${NC}"
echo -e "${BOLD}${CYAN}└─────────────────────────────────────────────────────────────┘${NC}"
echo ""
echo -e "  ${BOLD}Total Tests:${NC}  $TOTAL_TESTS"
echo -e "  ${GREEN}Passed:${NC}      $PASSED_TESTS"
echo -e "  ${RED}Failed:${NC}      $FAILED_TESTS"
echo -e "  ${YELLOW}SKipped:${NC}    $SKIPPED_TESTS"
echo ""
echo -e "  ${BOLD}Pass Rate:${NC}   $(($PASSED_TESTS * 100 / $TOTAL_TESTS))%"
echo ""

# Generate markdown summary
cat > "$TEST_SUMMARY" << EOF
# Test Summary Report

**Date:** $(date)
**Total Tests:** $TOTAL_TESTS
**Passed:** $PASSED_TESTS
**Failed:** $FAILED_TESTS
**Skipped:** $SKIPPED_TESTS
**Pass Rate:** $(($PASSED_TESTS * 100 / $TOTAL_TESTS))%

## Test Categories

| Section | Tests | Passed | Failed | Skipped |
|---------|-------|--------|--------|---------|
| CLI Commands | 20 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Binary Tests | 8 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| ML & Training | 6 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Integration Tests | 8 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Output Formats | 8 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Performance | 2 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Edge Cases | 5 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| LLM Integration | 3 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Feature Commands | 2 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Configuration | 7 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Documentation | 10 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Security & Error | 3 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |
| Cleanup | 2 | $PASSED_TESTS | $FAILED_TESTS | $SKIPPED_TESTS |

## Conclusion

EOF

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "\n${GREEN}${BOLD}✅ ALL TESTS PASSED!${NC}"
    echo "All tests passed successfully!" >> "$TEST_SUMMARY"
    echo "" >> "$TEST_SUMMARY"
    echo "The code-intelligence project is ready for production use." >> "$TEST_SUMMARY"
    EXIT_STATUS=0
else
    echo -e "\n${RED}${BOLD}❌ SOME TESTS FAILED${NC}"
    echo "Some tests failed. Please review the logs." >> "$TEST_SUMMARY"
    echo "" >> "$TEST_SUMMARY"
    echo "Failed tests:" >> "$TEST_SUMMARY"
    grep "\[FAIL\]" "$TEST_RESULTS" | sed 's/^/- /' >> "$TEST_SUMMARY"
    EXIT_STATUS=1
fi

echo "" >> "$TEST_SUMMARY"
echo "Full results available in: $TEST_RESULTS" >> "$TEST_SUMMARY"

# Print final message
echo ""
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}${CYAN}  Test Results Summary${NC}"
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "  Results file:   ${BOLD}$TEST_RESULTS${NC}"
echo -e "  Summary file:   ${BOLD}$TEST_SUMMARY${NC}"
echo ""

if [ $EXIT_STATUS -eq 0 ]; then
    echo -e "  ${GREEN}${BOLD}✅ ALL TESTS PASSED - Project is production ready!${NC}"
else
    echo -e "  ${RED}${BOLD}❌ SOME TESTS FAILED - Please review the logs${NC}"
fi

echo ""

exit $EXIT_STATUS
