# User Guide

## Overview

`code-intelligence` is a CLI platform for detecting dead code with high precision, eliminating structural duplication, and mapping polyglot call graphs. This guide covers setup, daily CLI workflows, terminal dashboard navigation, and advanced ML configurations.

---

## Quick Start

### 1. Installation

#### Automated Script (Recommended)

Pre-built standalone binaries include all machine learning models compiled directly into the binary:

- **Linux & macOS:**
  ```bash
  curl -fsSL https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.sh | bash
  ```

- **Windows (PowerShell as Admin):**
  ```powershell
  irm https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.ps1 | iex
  ```

- **Windows (CMD / Batch):**
  Download and run [`install.bat`](https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.bat) as Administrator.

#### Build from Source (Cargo)

```bash
git clone https://github.com/neontoshi/code-intelligence.git
cd code-intelligence
cargo install --path .
```

Verify the installation:

```bash
ci --version
```

### 2. First Analysis

Navigate to your target project and run a scan:

```bash
cd ~/my-project
ci analyze .
```

Review the candidates in the terminal list, or launch the interactive TUI:

```bash
ci list
# Or launch the interactive terminal UI:
ci dashboard .
```

---

## Core Commands

### Analysis

#### `ci analyze`

Scan a codebase for unreachable, unused, and dead functions, types, and modules.

```bash
# Standard analysis (uses embedded ML models by default)
ci analyze .

# Specify a custom confidence threshold (0.0 - 1.0)
ci analyze . --threshold 0.85

# Enable disk caching for incremental rescans
ci analyze . --cache

# Include Git history metrics (churn, author recency)
ci analyze . --git

# Run with optional LLM summaries
ci analyze . --llm
```

| Option | Description | Default |
|--------|--------------|---------|
| `--threshold` | Confidence threshold for dead classification | `0.92` |
| `--cache` | Enable AST file disk caching | `false` |
| `--cache-dir` | Custom directory for cache storage | `.code-intelligence-cache` |
| `--git` | Extract commit frequencies and author recency | `false` |
| `--llm` | Trigger function summaries and bug auditing | `false` |
| `--model` | Path to a custom `.bin` model file (overrides the embedded model) | *Embedded* |
| `--verbose` | Output detailed progress logs | `false` |

---

#### `ci dedup`

Find structural and semantic duplicate functions across the codebase.

```bash
# Standard structural deduplication
ci dedup .

# Run with the embedded ML pair classifier
ci dedup . --ml

# Set similarity threshold (0.0 - 1.0)
ci dedup . --threshold 0.80 --ml
```

| Option | Description | Default |
|--------|--------------|---------|
| `--threshold` | Similarity threshold for clone detection | `0.85` |
| `--ml` | Use the 101-feature ML duplicate classifier | `false` |
| `--duplicate-model` | Custom duplicate model path override | *Embedded* |

---

### Outcome Tracking & Triage

`code-intelligence` records removal outcomes and false-positive dismissals in `.code-intelligence-outcomes.json` to continuously refine accuracy.

#### `ci list`

List detected dead code candidates.

```bash
# List all pending candidates
ci list

# List all candidates, including resolved/whitelisted
ci list --all
```

#### `ci remove`

Mark a function as confirmed dead / deleted in the codebase.

```bash
ci remove process_data --commit a1b2c3d
```

#### `ci keep`

Mark a function as an intentional keep / false positive, with an audit reason.

```bash
ci keep setup_test_db "Invoked dynamically by integration suite"
```

#### `ci stats`

Inspect current removal rates and false-positive metrics.

```bash
ci stats --detailed
```

---

### Visualization & Reports

#### `ci report`

Export analysis summaries to various formats.

```bash
# Markdown summary
ci report . --format markdown --output report.md

# Structured JSON export
ci report . --format json --output report.json

# Full report with compressed semantic context
ci report . --format full --output summary.md
```

#### `ci graph`

Generate interactive call-graph visualizations as standalone HTML.

```bash
# Interactive D3.js node-link call graph
ci graph . --mode interactive --output call_graph.html

# Circular layer architectural overview
ci graph . --mode overview --output architecture.html
```

---

### Interactive Dashboard

Launch the full-screen terminal user interface (TUI) built with Ratatui:

```bash
ci dashboard .
```

| Key | Action |
|-----|--------|
| `Tab` / `Right` / `l` | Switch to next tab |
| `BackTab` / `Left` / `h` | Switch to previous tab |
| `Down` / `j` & `Up` / `k` | Scroll down / up through the candidate list |
| `g` / `G` | Jump to top / bottom of list |
| `d` | Mark the selected function as confirmed Dead |
| `f` | Mark the selected function as False Positive (prompts for reason) |
| `s` | Defer candidate review |
| `Enter` | Expand evidence and signal breakdown |
| `q` / `Esc` | Quit dashboard |

---

## Configuration Management

Manage default thresholds and provider preferences in `~/.config/code-intelligence/config.toml`:

```bash
# Update decision threshold
ci config set threshold 0.85

# Configure optional LLM providers
ci config set llm_provider ollama
ci config set llm_model phi:2.7b

# Inspect all configurations
ci config list
```

---

## ML Training & Calibration

All release binaries ship with embedded models, but you can train custom weights on proprietary repositories:

```bash
# 1. Export AST and graph feature vectors from codebases
ci export ~/my-repo --output data/raw_features.json

# 2. Merge, deduplicate, and split datasets into train/val/test splits
ci merge --input "data/*.json" --dedup --output combined.json
ci split --input combined.json --output-dir data/

# 3. Train a linear classifier
ci train --train-data data/train.json --val-data data/val.json --output custom_model.bin

# 4. Calibrate probabilities (temperature scaling)
ci calibrate --model custom_model.bin --val-data data/val.json --output custom_calibrated.bin

# 5. Evaluate per-language F1 and accuracy
ci evaluate-lang --model custom_calibrated.bin --test-data data/test.json --detailed
```

---

## CI/CD Integration Examples

### GitHub Actions Workflow

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
      - uses: actions/checkout@v4

      - name: Install Code Intelligence
        run: |
          curl -fsSL https://raw.githubusercontent.com/neontoshi/code-intelligence/main/install.sh | bash

      - name: Run Analysis Gate
        run: |
          ci analyze . --format json --output dead_code_report.json --threshold 0.85
```

> **Note:** the original step ran `ci ci . --format json ...` — same doubled-subcommand pattern flagged in `deployment.md`. Corrected to `ci analyze` to match every other invocation in this guide.

### Git Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/usr/bin/env bash
if ci stats . 2>/dev/null | grep -q "Pending: [1-9]"; then
    echo "❌ Commit rejected: Pending dead code reviews exist."
    echo "   Run 'ci list' to inspect or 'ci keep <name> \"reason\"' to whitelist."
    exit 1
fi
```

---

## Recommended Threshold Guidelines

| Project Type | Recommended Threshold | Rationale |
|---------------|-------------------------|-----------|
| **Public Libraries / SDKs** | `0.90–0.95` | Minimizes false positives on exported, uncalled public APIs |
| **Backend Services / Apps** | `0.80–0.85` | Balanced precision and recall for internal business logic |
| **Monolith Cleanup Sprints** | `0.70–0.75` | Aggressive pruning mode to surface subtle dead branches |
