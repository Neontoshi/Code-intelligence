# Code Intelligence

[![Build Status](https://img.shields.io/github/actions/workflow/status/neontoshi/Code-intelligence/ci.yml?branch=main)](https://github.com/neontoshi/Code-intelligence/actions)
[![Test Coverage](https://img.shields.io/codecov/c/github/neontoshi/Code-intelligence)](https://codecov.io/gh/neontoshi/Code-intelligence)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![Model Accuracy](https://img.shields.io/badge/accuracy-95.3%25-brightgreen)](docs/evaluation_report.md)
[![Precision](https://img.shields.io/badge/precision-96.8%25-brightgreen)](docs/evaluation_report.md)

**Semantic Code Intelligence Engine for AI Dead Code Detection, Duplicate Detection, and Code Analysis**

[![CI](https://github.com/neontoshi/Code-intelligence/actions/workflows/ci.yml/badge.svg)](https://github.com/neontoshi/Code-intelligence/actions/workflows/ci.yml)
[![Documentation](https://img.shields.io/badge/docs-passing-brightgreen)](docs/)


`code-intelligence` is a fast, ML-powered static and dynamic semantic analysis platform designed to inspect, optimize, and map complex codebases. It features full AST analysis across multiple languages, ML-driven dead code detection, semantic duplicate elimination, call/import/type graph visualization, interactive terminal dashboards, and local/cloud LLM analysis.

---

## ⚡ Key Highlights & Capabilities

* **Unified Verdict Engine**: Combines static reachability analysis, fan-in/fan-out graph metrics, dynamic reference detection, and calibrated ML models to determine if code is `Dead`, `Alive`, or needs review.


* **Multi-Language AST Parsing**: Full Tree-Sitter support for **Rust, TypeScript, JavaScript, Python, Go, and Java**.


* **Safe False-Positive Filtering**: Built-in awareness for trait implementations, public framework decorators (React hooks/components, FastAPI/Flask routes, Spring annotations, NestJS controllers), and lifecycle entry points.


* **Duplicate Code Detection**: Identifies identical and structurally similar code blocks using MinHash, AST hashing, and ML-based duplicate classification to suggest refactorings and estimate token savings.


* **Interactive Graphs & UI**:
* Full detailed interactive call graphs in HTML.


* Circular architectural layer overviews designed for non-technical walk-throughs.


* Terminal UI Dashboard (Ratatui/Crossterm) for real-time review, status tracking, and decision management.




* **LLM Integration**: Works with local (Ollama) and cloud providers (OpenAI, Anthropic) to generate documentation, summarize function logic, and flag code smells.


* **Outcome Management**: Built-in ledger tracking (`.code-intelligence-outcomes.json`) to confirm removals, track false positives, and continuously improve training datasets.



---

## 📦 Installation

### 1. Prerequisites

* [Rust & Cargo](https://rustup.rs/) (edition 2021)


* (Optional) [Ollama](https://ollama.com/) running locally for offline LLM features



### 2. Build & Install Everything

To install the `ci` binary and all supporting evaluation and training tools to `~/.cargo/bin`:

```bash
git clone https://github.com/neontoshi/Code-intelligence
cd code-intelligence
cargo install --path .

```

### 3. Verify Installation

```bash
ci --version
ci --help

```

---

## 🚀 Quick Start Guide

### Step 1: Configure Default Settings

Set your default model and threshold in your global configuration (`~/.config/code-intelligence/config.toml`):

```bash
# Point to your calibrated dead code model
ci config set model models/dead_code_model_v2.bin

# Set the default classification threshold (0.0 - 1.0)
ci config set threshold 0.55

# Set preferred local/cloud LLM provider
ci config set llm_provider ollama
ci config set llm_model phi:2.7b

```

### Step 2: Analyze a Project

Navigate to any target codebase and trigger the analysis:

```bash
cd ~/Documents/your-project
ci analyze

```

### Step 3: Inspect & Manage Dead Code

```bash
# List all pending dead functions
ci list

# Review outcome statistics
ci stats --detailed

# Mark a function as removed after deleting it in your editor
ci remove unused_helper_function --commit abc1234

# Mark a function as a false positive so it won't be flagged again
ci keep renderCustomView "Required by third-party plugin"

```

---

## 🛠️ CLI Reference (`ci`)

The primary executable `ci` provides commands for code analysis, review, model operations, and reporting:

### 1. Code Analysis & Inspection

| Command | Description | Example |
| --- | --- | --- |
| `ci analyze [path]` | Scan project for dead functions, types, and modules

 | `ci analyze ~/project --threshold 0.55 --git`<br> |
| `ci dedup [path]` | Find identical and structurally duplicate functions

 | `ci dedup . --threshold 0.85 --ml`<br> |
| `ci graph [path]` | Generate HTML graph visualization (`interactive` or `overview`)

 | `ci graph . --mode overview --output map.html`<br> |
| `ci llm [path]` | Run deep semantic review and bug scan via LLM

 | `ci llm . --provider openai --model gpt-4`<br> |
| `ci dashboard [path]` | Launch interactive terminal UI (Ratatui)

 | `ci dashboard .`<br> |

### 2. Outcome Management & Tracking

| Command | Description | Example |
| --- | --- | --- |
| `ci list [path]` | Display detected dead code candidates

 | `ci list --all`<br> |
| `ci remove <name>` | Mark dead candidate as deleted in the repo

 | `ci remove processOrder --commit 8f3d1b`<br> |
| `ci keep <name> "<reason>"` | Mark candidate as false positive / intentionally kept

 | `ci keep handlePing "Used by health check"`<br> |
| `ci update <id> <action>` | Update verdict by specific unique ID

 | `ci update auth_1710928 removed`<br> |
| `ci stats [path]` | View removal rates and false-positive metrics

 | `ci stats --detailed`<br> |
| `ci report [path]` | Generate markdown, JSON, or HTML analysis summaries

 | `ci report --format markdown --output report.md`<br> |

### 3. ML Training, Calibration & Experimentation

| Command | Description | Example |
| --- | --- | --- |
| `ci train` | Train a linear classifier for dead code detection

 | `ci train --data data/train.json --precision 0.95`<br> |
| `ci train-duplicate` | Train a classifier for code duplicate identification

 | `ci train-duplicate data/pairs.json --output dup_model.bin`<br> |
| `ci calibrate` | Calibrate confidence scores (temperature scaling, isotonic)

 | `ci calibrate --method temperature --val-data data/val.json`<br> |
| `ci tune` | Find the optimal decision threshold for target precision

 | `ci tune --precision 0.99`<br> |
| `ci compare` | Compare accuracy and F1 across multiple model configurations

 | `ci compare --train-data data/train.json`<br> |
| `ci features` | Display top differentiating features per programming language

 | `ci features --data combined_training.json`<br> |
| `ci ablation` | Run feature ablation studies to measure feature importance

 | `ci ablation --output-dir ./ablation_results`<br> |
| `ci evaluate-lang` | Evaluate precision, recall, and false-positive rate per language

 | `ci evaluate-lang --model model.bin --test-data data/test.json`<br> |

### 4. Training Data Management

| Command | Description | Example |
| --- | --- | --- |
| `ci export [path]` | Extract AST and graph feature vectors into training JSON

 | `ci export . --output repo_features.json`<br> |
| `ci merge` | Deduplicate and split repo datasets into train/val/test splits

 | `ci merge --input "training_data/*.json" --dedup`<br> |
| `ci collect` | Clone public repositories and generate bulk training sets

 | `ci collect --max-repos 25`<br> |
| `ci verify` | Produce a Markdown review checklist for dead candidates

 | `ci verify --data data/val.json --output checklist.md`<br> |
| `ci self-analyze` | Run full analysis pipeline on the `code-intelligence` codebase

 | `ci self-analyze --format full`<br> |

### 5. Global Configuration

| Command | Description | Example |
| --- | --- | --- |
| `ci config set <key> <val>` | Update setting (`model`, `threshold`, `llm_provider`, `llm_model`, `verbose`)

 | `ci config set threshold 0.60`<br> |
| `ci config get <key>` | Read active config setting

 | `ci config get model`<br> |
| `ci config list` | Show defaults and per-project configurations

 | `ci config list`<br> |

---

## 🧰 Standalone Binaries

In addition to the `ci` driver, specialized binaries are available directly via Cargo:

```bash
# Core analyzers
cargo run --release --bin dead_code_check -- ./path/to/project --model models/dead_code_model_v2.bin[cite: 1]
cargo run --release --bin dedup_check -- ./path/to/project --threshold 0.80[cite: 1]
cargo run --release --bin dead_code_dashboard -- ./path/to/project[cite: 1]

# ML Pipeline & Data Engineering
cargo run --release --bin collect_training_data[cite: 1]
cargo run --release --bin merge_all_training_data[cite: 1]
cargo run --release --bin train_model -- --train-data data/train.json --target-precision 0.95[cite: 1]
cargo run --release --bin calibrate_model -- --model model.bin --val-data data/val.json[cite: 1]
cargo run --release --bin tune_threshold -- --model model.bin --val-data data/val.json --target-precision 0.99[cite: 1]
cargo run --release --bin evaluate_per_language -- --model model.bin --test-data data/test.json[cite: 1]
cargo run --release --bin feature_ablation -- --train-data data/train.json --val-data data/val.json[cite: 1]

```

---

## 🖥️ Terminal Interactive Dashboard (`ci dashboard`)

Launch a full-screen terminal UI built with `ratatui`:

```bash
ci dashboard ~/Documents/my-project

```

### Dashboard Views & Navigation:

* **Summary**: High-level metrics, health status, and estimated removable lines of code.


* **Charts**: Visual distribution of dead code across modules, languages, and confidence intervals.


* **List**: Sortable table of dead function candidates with line numbers and confidence scores.


* **By File**: File-by-file grouped breakdown of dead functions and types.


* **Priority**: Ordered step-by-step removal plan minimizing breakage risk.


* **History**: Audit log of confirmed removals, false-positive dismissals, and user actions.



**Keybindings**:

* `Tab` / `Right` / `l`: Next tab


* `BackTab` / `Left` / `h`: Previous tab


* `Down` / `j` & `Up` / `k`: Scroll list items


* `g` / `G`: Jump to top / bottom


* `q` / `Esc`: Exit dashboard



---

## 🌐 Call Graph Visualizations (`ci graph`)

`code-intelligence` provides two visual graph formats rendered entirely in HTML/SVG using D3.js:

```bash
# 1. Detailed Interactive View (for engineering analysis)
ci graph . --mode interactive --output call_graph.html

# 2. High-Level Architectural View (for presentations and non-tech stakeholders)
ci graph . --mode overview --output call_graph_overview.html

```

---

## 🔄 CI/CD Automation Examples

### 1. GitHub Actions (Dead Code Gate & Report)

```yaml
name: Dead Code & Optimization Gate

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
          override: true

      - name: Install Code Intelligence
        run: |
          git clone https://github.com/neontoshi/code-intelligence.git /tmp/ci
          cd /tmp/ci && cargo install --path .

      - name: Run Dead Code Analysis
        run: |
          ci config set model /tmp/ci/models/dead_code_model_v2.bin
          ci config set threshold 0.70
          ci analyze . --cache

      - name: Generate Reports
        run: |
          ci report . --format markdown --output dead_code_report.md
          ci report . --format json --output dead_code_report.json

      - name: Upload Analysis Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: code-intelligence-report
          path: |
            dead_code_report.md
            dead_code_report.json

```

### 2. Pre-Commit Hook (Prevent Committing Dead Code)

Add to `.git/hooks/pre-commit` and make executable (`chmod +x .git/hooks/pre-commit`):

```bash
#!/usr/bin/env bash
if ci stats . 2>/dev/null | grep -q "Pending: [1-9]"; then
    echo "❌ Commit rejected: Pending dead code findings detected."
    echo "   Run 'ci list' to review findings."
    echo "   Run 'ci remove <name>' if deleted, or 'ci keep <name> \"<reason>\"' to whitelist."
    exit 1
fi

```

---

## 🏗️ Architecture & Project Layout

```
code-intelligence/
├── src/
│   ├── analysis/             # Analysis logic (dead code, complexity, dynamic refs, reachability)
│   │   ├── dead_code/        # Scorer, whitelist, type/module analysis, impact estimators
│   │   ├── dynamic_refs.rs   # Reflection, framework callback, and string-dispatch detection
│   │   ├── roots.rs          # Root detection & BFS reachability analysis
│   │   └── verdict.rs        # Verdict decision engine combining static & ML signals
│   ├── bin/                  # CLI tool (`ci`), dashboard, and ML training/eval binaries
│   ├── engine/               # Indexer, file walking, disk caching, pipeline stages
│   ├── graph/                # Call, dependency, import, and type graphs (Petgraph)
│   ├── llm/                  # Providers: Ollama, OpenAI, Anthropic, Mock
│   ├── ml/                   # Classifier, feature schema, calibration, serialization
│   ├── optimize/             # Deduplication, MinHash, token estimation, compression
│   ├── output/               # Markdown, JSON, RAG, and interactive/overview HTML graphs
│   └── parser/               # Tree-sitter parsers & semantic analyzers
├── models/                   # Pretrained and calibrated .bin models
└── data/                     # Train, validation, and test datasets

```

---

## 🧪 Testing & Benchmarks

```bash
# Run unit and integration tests
cargo test

# Run integration tests specifically
cargo test --test integration

# Run criterion compression and deduplication benchmarks
cargo bench

```

---

## 📄 License

This project is licensed under the **MIT License**.
