## Document 5: `docs/user_guide.md`

```markdown
# User Guide

## Overview

`code-intelligence` is a CLI tool for detecting dead code, finding duplicates, and analyzing codebases. This guide covers everything you need to know to use it effectively.

---

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/neontoshi/Code-intelligence
cd code-intelligence

# Build and install
cargo install --path .

# Verify installation
ci --version
```

### First Analysis

```bash
# Navigate to your project
cd ~/my-project

# Run analysis
ci analyze .

# View results
ci list
```

---

## Core Commands

### Analysis

#### `ci analyze`

Analyze a project for dead code.

```bash
# Basic analysis
ci analyze .

# With custom threshold
ci analyze . --threshold 0.85

# With LLM analysis
ci analyze . --llm

# With Git history
ci analyze . --git

# With cache
ci analyze . --cache

# Verbose output
ci analyze . --verbose
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--threshold` | Confidence threshold (0.0-1.0) | 0.92 |
| `--llm` | Enable LLM analysis | false |
| `--git` | Enable Git analysis | false |
| `--cache` | Enable disk cache | false |
| `--cache-dir` | Cache directory | `.code-intelligence-cache` |
| `--verbose` | Verbose output | false |

#### `ci dedup`

Find duplicate code.

```bash
# Find duplicates
ci dedup .

# With custom threshold
ci dedup . --threshold 0.80

# With ML model
ci dedup . --ml
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--threshold` | Similarity threshold (0.0-1.0) | 0.85 |
| `--ml` | Use ML model | false |

---

### Outcome Management

#### `ci list`

List dead functions found.

```bash
# List pending dead functions
ci list

# List all (including resolved)
ci list --all
```

**Options:**

| Option | Description |
|--------|-------------|
| `--all` | Show all including resolved |

#### `ci remove`

Mark a function as removed.

```bash
# Remove by name (partial match)
ci remove process_data

# With commit hash
ci remove process_data --commit abc123
```

**Options:**

| Option | Description |
|--------|-------------|
| `--commit` | Git commit hash |

#### `ci keep`

Mark a function as a false positive.

```bash
ci keep setup_test "Used by integration tests"
```

#### `ci update`

Update a verdict by ID.

```bash
# Mark as removed
ci update abc123 removed --commit abc123

# Mark as false positive
ci update abc123 false-positive "Used by tests"
```

#### `ci stats`

Show outcome statistics.

```bash
# Basic stats
ci stats

# Detailed stats
ci stats --detailed
```

**Options:**

| Option | Description |
|--------|-------------|
| `--detailed` | Show detailed breakdown |

---

### Reporting

#### `ci report`

Generate a report.

```bash
# Markdown report
ci report . --format markdown

# JSON report
ci report . --format json --output report.json

# Full report with LLM
ci report . --format full --llm
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--format` | markdown, json, html, full | markdown |
| `--output` | Output file | auto-generated |
| `--llm` | Include LLM analysis | false |

#### `ci graph`

Generate a call graph visualization.

```bash
# Interactive graph (for engineers)
ci graph . --mode interactive

# Overview graph (for non-technical)
ci graph . --mode overview --output architecture.html
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--mode` | interactive, overview | interactive |
| `--output` | Output HTML file | call_graph.html |

---

### LLM Analysis

#### `ci llm`

Run LLM-powered analysis.

```bash
# With Ollama
ci llm . --provider ollama

# With OpenAI
ci llm . --provider openai --model gpt-4 --api-key $OPENAI_API_KEY

# With custom temperature
ci llm . --temperature 0.5
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--provider` | ollama, openai, anthropic | ollama |
| `--model` | Model name | phi:2.7b |
| `--api-key` | API key | - |
| `--temperature` | Temperature (0.0-1.0) | 0.3 |
| `--max-tokens` | Max output tokens | 1000 |

---

### Dashboard

#### `ci dashboard`

Open interactive terminal UI.

```bash
ci dashboard .
```

**Controls:**

| Key | Action |
|-----|--------|
| `Tab` / `→` | Next tab |
| `BackTab` / `←` | Previous tab |
| `↓` / `j` | Scroll down |
| `↑` / `k` | Scroll up |
| `g` | Jump to top |
| `G` | Jump to bottom |
| `d` | Mark as dead |
| `f` | Mark as false positive |
| `s` | Defer |
| `q` / `Esc` | Quit |
| `Enter` | Show evidence |

---

### Configuration

#### `ci config`

Manage configuration.

```bash
# Set values
ci config set model models/dead_code_model_v2.bin
ci config set threshold 0.85
ci config set verbose true
ci config set llm_provider ollama
ci config set llm_model phi:2.7b

# Get values
ci config get model
ci config get threshold

# List all
ci config list
```

**Config Keys:**

| Key | Description | Default |
|-----|-------------|---------|
| `model` | ML model path | - |
| `threshold` | Confidence threshold | 0.92 |
| `verbose` | Verbose output | false |
| `llm_provider` | LLM provider | ollama |
| `llm_model` | LLM model | phi:2.7b |

---

## ML & Training Commands

### Training

#### `ci train`

Train the ML model.

```bash
# Basic training
ci train --data data/train.json

# With validation
ci train --data data/train.json --val-data data/val.json

# With custom output
ci train --output model.bin --precision 0.98
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--data` | Training data | data/train.json |
| `--val-data` | Validation data | - |
| `--output` | Output model | model.bin |
| `--precision` | Target precision | 0.95 |

#### `ci train-duplicate`

Train duplicate detection model.

```bash
ci train-duplicate data/pairs.json --output dup_model.bin
```

#### `ci calibrate`

Calibrate a trained model.

```bash
# Temperature scaling
ci calibrate --model model.bin --data data/val.json

# Histogram binning
ci calibrate --method histogram

# Isotonic regression
ci calibrate --method isotonic
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--model` | Input model | model.bin |
| `--data` | Validation data | data/val.json |
| `--output` | Output model | model_calibrated.bin |
| `--method` | temperature, histogram, isotonic | temperature |

#### `ci tune`

Tune confidence threshold.

```bash
ci tune --model model.bin --data data/val.json --precision 0.99
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--model` | Model file | model.bin |
| `--data` | Validation data | data/val.json |
| `--precision` | Target precision | 0.99 |

---

### Evaluation

#### `ci evaluate-lang`

Evaluate model per language.

```bash
# Basic evaluation
ci evaluate-lang --model model.bin --test-data data/test.json

# With detailed metrics
ci evaluate-lang --detailed

# With validation
ci evaluate-lang --val-data data/val.json
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--model` | Model file | model.bin |
| `--test-data` | Test data | data/test.json |
| `--val-data` | Validation data | - |
| `--detailed` | Show detailed metrics | false |

#### `ci compare`

Compare different ML models.

```bash
ci compare --train-data data/train.json --val-data data/val.json --test-data data/test.json
```

#### `ci features`

Analyze feature importance per language.

```bash
ci features --data combined_training.json
```

#### `ci ablation`

Run feature ablation study.

```bash
ci ablation --train-data data/train.json --val-data data/val.json --output ablation_results
```

---

### Data Management

#### `ci export`

Export training data from a project.

```bash
ci export . --output training_data.json
```

#### `ci merge`

Merge training data files.

```bash
# Merge all JSON files
ci merge --input "training_data/*.json"

# With deduplication
ci merge --dedup
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--input` | Input glob | training_data/*.json |
| `--output` | Output file | combined_training.json |
| `--dedup` | Deduplicate examples | false |

#### `ci collect`

Collect training data from repositories.

```bash
# Use default list
ci collect

# Custom repositories
ci collect https://github.com/rust-lang/rust.git https://github.com/tokio-rs/tokio.git
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--output` | Output directory | training_data |
| `--max-repos` | Max repositories | 50 |

#### `ci split`

Split data by repository.

```bash
ci split --input combined_training.json --output-dir data
```

**Options:**

| Option | Description | Default |
|--------|-------------|---------|
| `--input` | Input file | combined_training.json |
| `--output-dir` | Output directory | data |
| `--train-ratio` | Train ratio | 0.7 |
| `--val-ratio` | Validation ratio | 0.15 |
| `--test-ratio` | Test ratio | 0.15 |

---

### Advanced Commands

#### `ci verify`

Generate review checklist.

```bash
ci verify --data data/val.json --output review_checklist.md
```

#### `ci hard-negatives`

Generate hard-negative dataset.

```bash
ci hard-negatives . --count 100 --min-confidence 0.7
```

#### `ci temporal`

Run temporal evaluation.

```bash
ci temporal --model model.bin --test-data data/test.json --windows 5
```

#### `ci verify-ground-truth`

Verify ground truth dataset.

```bash
# Generate review file
ci verify-ground-truth . --output verified_dataset.json

# Interactive mode
ci verify-ground-truth . --interactive
```

#### `ci calibration`

Run calibration analysis.

```bash
ci calibration --model model.bin --val-data data/val.json --test-data data/test.json --report
```

#### `ci self-analyze`

Analyze code-intelligence itself.

```bash
ci self-analyze --format markdown
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: Dead Code Check

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Code Intelligence
        run: |
          git clone https://github.com/neontoshi/code-intelligence.git /tmp/ci
          cd /tmp/ci && cargo install --path .

      - name: Run Analysis
        run: |
          ci config set model /tmp/ci/models/dead_code_model_v2.bin
          ci analyze . --threshold 0.85

      - name: Generate Report
        run: ci report . --format json --output dead_code_report.json

      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: dead-code-report
          path: dead_code_report.json
```

### Git Pre-commit Hook

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit

if ci stats . 2>/dev/null | grep -q "Pending: [1-9]"; then
    echo "❌ Commit rejected: Pending dead code found"
    echo "   Run 'ci list' to review"
    echo "   Run 'ci remove <name>' if deleted"
    echo "   Run 'ci keep <name> \"reason\"' to whitelist"
    exit 1
fi
```

---

## Workflow Examples

### Daily Development

```bash
# 1. Write code
git add .
git commit -m "Add new feature"

# 2. Check for dead code
ci analyze .

# 3. Review results
ci list

# 4. Remove dead code
ci remove old_helper --commit $(git rev-parse HEAD)

# 5. Commit removal
git add .
git commit -m "Remove dead code"

# 6. Track progress
ci stats
```

### PR Review

```bash
# 1. Analyze PR branch
git checkout pr-branch
ci analyze . --threshold 0.85

# 2. Generate report for reviewers
ci report . --format markdown --output dead_code_pr.md

# 3. Include in PR description
cat dead_code_pr.md
```

### Refactoring Sprint

```bash
# 1. Get baseline
ci analyze .
ci stats --detailed > before.md

# 2. Find high-priority dead code
ci list --all | grep -E "Confidence: (9[0-9]|100)"

# 3. Remove in batches
ci remove func1 --commit hash1
ci remove func2 --commit hash2

# 4. Verify progress
ci stats --detailed > after.md

# 5. Compare
diff before.md after.md
```

---

## Best Practices

### 1. Use Cache
```bash
# First run (slow)
ci analyze . --cache

# Subsequent runs (fast)
ci analyze . --cache
```

### 2. Set Appropriate Threshold

| Project Type | Recommended Threshold |
|--------------|----------------------|
| **Production** | 0.85-0.92 |
| **Library** | 0.80-0.85 |
| **Internal Tool** | 0.75-0.80 |
| **CI/CD Gate** | 0.85-0.90 |

### 3. Review Before Removing

Always review dead code candidates before removing:

```bash
# Show details
ci list

# Get explanation
ci report . --format markdown

# Manual review
ci dashboard .
```

### 4. Track Outcomes

```bash
# Track what you remove
ci remove func1 --commit $(git rev-parse HEAD)

# Track false positives
ci keep func2 "Used in tests"

# Monitor progress
ci stats
```

### 5. Regular Maintenance

```bash
# Weekly: Run analysis
ci analyze .

# Monthly: Review stats
ci stats --detailed

# Quarterly: Retrain model
ci collect
ci merge --dedup
ci split
ci train
ci calibrate
```

---

## Troubleshooting

### Common Issues

#### "Model not found"

```bash
# Configure model path
ci config set model models/dead_code_model_v2.bin

# Or download model
curl -L https://example.com/model.bin -o model.bin
```

#### "Out of memory"

```bash
# Use smaller threshold
ci analyze . --threshold 0.92

# Limit files
ci analyze . --max-files 1000

# Disable expensive features
ci analyze . --no-cycle-detection
```

#### "Parser errors"

```bash
# Check file encoding
file -i file.rs

# Skip problematic files
ci analyze . --exclude "**/generated/*"
```

#### "LLM not available"

```bash
# Check Ollama
ollama list

# Pull model
ollama pull phi:2.7b

# Or disable LLM
ci analyze . --no-llm
```

---

## Advanced Tips

### Custom Whitelist

Create a whitelist file:

```json
// .code-intelligence-whitelist.json
{
  "functions": ["reflection_target", "ffi_export"],
  "patterns": ["^test_", "^bench_"],
  "files": ["**/generated/*", "**/protobuf/*"]
}
```

### Performance Optimization

```bash
# Use all cores
export RAYON_NUM_THREADS=8

# Use memory limit
export CI_MEMORY_LIMIT_MB=4096

# Use cache
ci analyze . --cache
```

### Custom Reports

```bash
# Generate JSON for custom processing
ci report . --format json | jq '.dead_functions[] | select(.confidence > 0.9)'

# Generate markdown for documentation
ci report . --format markdown --output dead_code.md

# Generate HTML for sharing
ci report . --format html --output dead_code.html
```

---

## Next Steps

- Read the [Architecture Guide](architecture.md) to understand internals
- Check [Evaluation Report](evaluation_report.md) for performance metrics
- See [API Documentation](api.md) for programmatic usage
- Review [Limitations](limitations.md) to understand what it can't do
```

---
