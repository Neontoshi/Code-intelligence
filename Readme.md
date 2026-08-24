# Code Intelligence

[![Build Status](https://img.shields.io/github/actions/workflow/status/neontoshi/Code-intelligence/ci.yml?branch=main)](https://github.com/neontoshi/Code-intelligence/actions)
[![Test Coverage](https://img.shields.io/codecov/c/github/neontoshi/Code-intelligence)](https://codecov.io/gh/neontoshi/Code-intelligence)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![Model Accuracy](https://img.shields.io/badge/accuracy-95.3%25-brightgreen)](docs/evaluation_report.md)
[![Precision](https://img.shields.io/badge/precision-96.8%25-brightgreen)](docs/evaluation_report.md)

**High-precision semantic code intelligence engine for dead code detection, structural deduplication, and polyglot call-graph mapping.**

`code-intelligence` is a fast, multi-language static and dynamic analysis platform that combines AST parsing, graph reachability algorithms, and calibrated machine learning models to detect dead code with high precision, eliminate duplicate logic, and optimize large polyglot codebases.

---

## ⚡ Key Highlights & Capabilities

- **Standalone Binary with Embedded ML**: Trained ML classifiers for dead code and duplicate detection are baked directly into the executable via compile-time embedding. No external model files or runtime downloads required.
- **Unified Verdict Engine**: Combines static BFS reachability analysis, fan-in/fan-out graph metrics, dynamic reference detection, and temperature-calibrated linear ML models to classify symbols into 5 confidence states (`DefinitelyAlive`, `ProbablyAlive`, `Unknown`, `ProbablyDead`, `DefinitelyDead`).
- **Native Multi-Language Support (10 Languages)**:
  - **Rust**: `impl` blocks, traits, operator overloads, FFI, and macros.
  - **TypeScript / TSX / JavaScript**: ES6 modules, barrel exports, React components, hooks (`use*`), and event handlers.
  - **Python**: Decorators (FastAPI, Flask, Pytest, Celery), magic dunder methods, `self.`/`cls.` calls, and `getattr()` string dispatches.
  - **Go**: Export capitalization, receiver methods (`func (r *Repo)`), `init()` hooks, `Test*`/`Benchmark*` suites, and reflection.
  - **Java**: Access modifiers, class methods, records, and Spring/Jakarta annotations (`@GetMapping`, `@Service`, `@Repository`).
  - **C# / .NET**: ASP.NET Core controllers, route attributes (`[HttpGet]`, `[HttpPost]`), MediatR handlers, and `Program.cs` entry points.
  - **Dart / Flutter**: Widget lifecycle methods (`build`, `initState`, `dispose`), state handlers, and `lib/main.dart` entry points.
  - **PHP**: Magic methods, Laravel/Symfony controller attributes, and dynamic execution (`call_user_func`).
  - **C++**: Destructors, special member functions, entry macros, and header file declarations.
- **Dynamic Reference Detection**: Tracks reflection calls, dynamic imports (`import()`, `require()`), IPC bridges (Tauri `invoke(...)`, Electron `ipcRenderer.send(...)`), and string dispatch routes.
- **Structural Duplicate Elimination**: Identifies duplicate blocks and code clones using MinHash, AST hashing, and ML pair classification to estimate token savings and suggest refactoring targets.
- **Outcome Management**: Built-in tracking ledger (`.code-intelligence-outcomes.json`) records removals and false-positive dismissals to continuously improve training datasets.
- **Interactive Terminal Dashboard**: Full-screen TUI built with Ratatui and Crossterm for live inspection, graph metrics, file-by-file categorization, and decision management.
- **Visual Graph Output**: Exports interactive D3.js call graphs and circular architectural layer overviews in standalone HTML.
- **Optional LLM Extensions**: Pluggable provider support (Ollama, OpenAI, Anthropic) for documentation generation, automated function summarization, and issue auditing.

---

## 📦 Installation

### Quick Install (Pre-built Binaries)

**Linux & macOS**
```bash
curl -fsSL https://raw.githubusercontent.com/neontoshi/Code-intelligence/main/install.sh | bash
```

**Windows (PowerShell as Administrator)**
```powershell
irm https://raw.githubusercontent.com/neontoshi/Code-intelligence/main/install.ps1 | iex
```

**Windows (Command Prompt / Batch)**

Download and run [`install.bat`](https://raw.githubusercontent.com/neontoshi/Code-intelligence/main/install.bat) as Administrator.

### Build from Source (Cargo)

**Prerequisites:**
- [Rust & Cargo](https://rustup.rs/) (1.70+)

```bash
git clone https://github.com/neontoshi/Code-intelligence.git
cd Code-intelligence
cargo install --path .
```

Verify installation:
```bash
ci --version
ci --help
```

---

## 🚀 Quick Start

### 1. Global Configuration

Set default model, decision threshold, and optional LLM preferences in `~/.config/code-intelligence/config.toml`:

```bash
# Point at a trained model (required before analyze/list/deadcode)
ci config set model ~/.local/share/code-intelligence/model.bin

# Set decision threshold (default: 0.92)
ci config set threshold 0.92

# Configure optional LLM provider
ci config set llm_provider ollama
ci config set llm_model phi:2.7b

# Inspect current settings
ci config list
```

### 2. Scan a Codebase

```bash
ci analyze ~/Documents/my-project
```

### 3. Launch the Interactive Dashboard

```bash
ci dashboard ~/Documents/my-project
```

---

## 🛠️ CLI Reference (`ci`)

### 1. Inspection & Analysis

| Command | Description | Example |
|---|---|---|
| `ci analyze [path]` | Full project scan: dead code + duplicates + high-impact functions in one pass | `ci analyze . --threshold 0.92 --git` |
| `ci list [path]` | Quick table of dead (and optionally unknown) functions | `ci list . --all` |
| `ci deadcode [path]` | Detailed dead-code report with priority removal order | `ci deadcode . --threshold 0.9 --output deadcode.md` |
| `ci dedup [path]` | Detailed duplicate/clone detection report | `ci dedup . --threshold 0.85 --ml` |
| `ci graph [path]` | Generate an interactive or overview HTML call graph | `ci graph . --mode interactive --output graph.html` |
| `ci dashboard [path]` | Launch the interactive terminal UI (Ratatui) | `ci dashboard .` |
| `ci check [path]` | CI/CD gate mode — exits non-zero on failed thresholds | `ci check . --max-dead 5 --fail-on-dead` |

### 2. Outcome Tracking & Management

| Command | Description | Example |
|---|---|---|
| `ci remove <name>` | Mark a dead-code candidate as removed in the outcome ledger | `ci remove processOrder --commit 8f3d1b` |
| `ci keep <name> "<reason>"` | Mark a candidate as a false positive / intentionally kept | `ci keep handlePing "Health check callback"` |
| `ci stats [path]` | View removal rates and false-positive metrics | `ci stats . --detailed` |
| `ci report [path]` | Export markdown, JSON, or HTML analysis summaries | `ci report . --format markdown --output report.md` |

### 3. Configuration

| Command | Description | Example |
|---|---|---|
| `ci config set <key> <value>` | Set a config value (`model`, `duplicate_model`, `threshold`, `verbose`, `llm_provider`, `llm_model`) | `ci config set threshold 0.9` |
| `ci config get <key>` | Read a single config value | `ci config get threshold` |
| `ci config list` | Print the full resolved config | `ci config list` |

> Advanced ML training/calibration commands (`train`, `train-duplicate`, `calibrate`, `tune`, `export`, `merge`, `collect`, `export-feedback`, `update`, `self-analyze`) are compiled behind the `advanced` feature flag and hidden from `--help` by default — see [Advanced / Model Training](#-advanced--model-training) below.

---

## 📖 Example Walkthrough

The example below uses a dummy project, **`widget-service`** (a fictional Rust/TS service), to show what a full run actually looks like end to end.

### Set up

```bash
$ cd ~/projects/widget-service
$ ci config set model ~/.config/code-intelligence/model.bin
✅ Model set to: /home/neon/.config/code-intelligence/model.bin
```

### Full analysis

```bash
$ ci analyze . --threshold 0.9 --git
🔍 Analyzing project: "."
============================================================

📊 Detected project type: rust-cargo

════════════════════════════════════════════════════════════
🔍 DEAD CODE ANALYSIS
════════════════════════════════════════════════════════════

📊 Dead Code Summary:
   Total functions: 842
   Dead functions: 17
   Alive functions: 803
   Unknown: 22
   Effective threshold: 0.90
   Dead code ratio: 2.0%
   Estimated LOC removable: 214

🎯 Dead Functions (Priority Order):
   #    Function                                 Confidence   Impact     LOC
   ---- ---------------------------------------- ------------ ---------- --------
   1    legacy_price_calculator                  🔴 98.4%      high       46
   2    unused_webhook_retry_v1                  🔴 96.1%      medium     28
   3    format_invoice_footer_old                🟠 87.9%      low        11

   Run `ci deadcode . --output report.md` for full details

════════════════════════════════════════════════════════════
🔄 DUPLICATE CODE ANALYSIS
════════════════════════════════════════════════════════════

📊 Duplicate Code Summary:
   Duplicate groups: 3
   Total token savings: ~1,240
   Confidence: 91.2%

🔍 Duplicate Groups:
   #    Type         Functions  Similarity      Savings
   ---- ------------ ---------- --------------- ---------------
   1    Structural   4          94.5%           ~520
   2    Exact        2          100.0%          ~310
   3    Structural   3          88.2%           ~410

   Run `ci dedup . --output report.md` for full details

════════════════════════════════════════════════════════════
🔥 IMPORTANT FUNCTIONS (High Impact)
════════════════════════════════════════════════════════════

   Top 10 most important functions (by call frequency):
   Function                                Importance   Callers
   ---------------------------------------- ------------ ----------
   🔥 handle_create_widget                  0.94         38
   🔥 validate_order_payload                0.81         26
   📌 build_response_envelope               0.63         19

════════════════════════════════════════════════════════════
💡 RECOMMENDATIONS
════════════════════════════════════════════════════════════

   1. 🧹 Remove 17 dead functions (214 LOC)
      → `ci deadcode . --output deadcode.md`
   2. 🔄 Refactor 3 duplicate groups
      → `ci dedup . --output dedup.md`
   3. 📊 Generate complete report
      → `ci report . --format markdown --output full_report.md`

============================================================

✅ Analysis complete!
```

### Reviewing and acting on findings

```bash
$ ci list . --all
🔍 Scanning for dead code in: "."
   (showing dead + unknown verdicts)

📊 Dead Code Summary:
════════════════════════════════════════════════════════════
   Total functions: 842
   Dead functions: 17
   Unknown functions: 22
   Dead code ratio: 2.0%

📋 Functions:

| # | Function                  | Verdict | Confidence | File               |
|---|----------------------------|---------|------------|--------------------|
| 1 | legacy_price_calculator    | Dead    | 98.4%      | pricing.rs         |
| 2 | unused_webhook_retry_v1    | Dead    | 96.1%      | webhooks.rs        |
| 3 | format_invoice_footer_old  | Dead    | 87.9%      | invoices.rs        |

💡 Commands:
   ci deadcode . --output report.md  - Full detailed report
   ci remove <name>                   - Mark as removed
   ci keep <name> "reason"           - Mark as false positive

$ ci remove legacy_price_calculator --commit 8f3d1b2
✅ Marked 'legacy_price_calculator' as removed

$ ci keep format_invoice_footer_old "Still called from the legacy billing cron until Q4 migration"
✅ Marked 'format_invoice_footer_old' as false positive

$ ci stats . --detailed

📊 Outcome Statistics for: "."

   Total flagged: 17
   Removed: 1 (5.9%)
   Kept (false positives): 1
   Pending: 15

📈 Detailed Feedback Stats:
   Total decisions: 2
   Feedback ratio: 11.8%
   False positive rate: 50.0%
```

### CI/CD gating

```bash
$ ci check . --max-dead 5 --threshold 0.9 --format summary
🤖 Running in CI mode for: "."
   Threshold: 0.90

📊 CI Report
===========
Project: .
Threshold: 0.90
Total Functions: 842
Dead Functions: 17
Dead Ratio: 2.0%
Status: ❌ FAIL

❌ Dead code count 17 exceeds limit 5
```

Wire this into GitHub Actions or any CI runner as a gate — a non-zero exit fails the build.

### Visualizing the call graph

```bash
$ ci graph . --mode interactive --output widget-service-graph.html
📊 Generating interactive call graph for: "."
✅ HTML saved to: "widget-service-graph.html"
   Functions: 842
   Edges: 2,105
```

### Interactive dashboard

```bash
$ ci dashboard .
📊 Opening dashboard for: "."
```
Opens a full-screen Ratatui TUI with **Summary**, **Charts**, **List**, **By File**, **Priority**, and **History** tabs.

**Keybindings**:
- `Tab` / `Right` / `l` — Next tab
- `BackTab` / `Left` / `h` — Previous tab
- `Down` / `j` & `Up` / `k` — Navigate list items
- `g` / `G` — Jump to top / bottom
- `d` — Mark candidate as confirmed Dead
- `f` — Mark candidate as False Positive (prompts for reason)
- `s` — Defer candidate review
- `q` / `Esc` — Exit dashboard

---

## 🏗️ Architecture

```
code-intelligence/
├── src/
│   ├── analysis/             # Core analysis logic (dead code, complexity, dynamic refs, reachability)
│   │   ├── dead_code/        # Scorer, filters, whitelist, type/module analysis, impact estimators
│   │   ├── verdict_source/   # Decision engine combining static heuristics, graph signals & ML
│   │   ├── dynamic_refs.rs   # AST-based reflection, IPC, and dynamic dispatch detection
│   │   ├── roots.rs          # Root entry point detection & BFS reachability computation
│   │   └── explainability.rs # Transparent evidence generators and risk assessment
│   ├── bin/                  # CLI tool (`ci`), TUI dashboard, and ML training/eval binaries
│   ├── engine/                # Parser coordinator, index builder, caching, and pipeline stages
│   ├── graph/                 # Call, dependency, import, and type graphs (Petgraph)
│   ├── llm/                   # Providers: Ollama, OpenAI, Anthropic, Mock
│   ├── ml/                    # Linear classifier, feature schema, temperature calibration
│   ├── optimize/               # Deduplication, MinHash, token estimation, compression
│   ├── output/                 # Markdown, JSON, RAG generators, interactive HTML visualizations
│   └── parser/                 # Tree-sitter multi-language AST extraction
├── models/                    # Pretrained and calibrated binary models (embedded into build)
└── data/                      # Training, validation, and test datasets
```

The `ci` binary itself (`src/bin/ci/`) is a thin CLI layer — `main.rs` dispatches into per-command modules (`analyze.rs`, `deadcode.rs`, `dedup.rs`, `list.rs`, `remove.rs`, `keep.rs`, `stats.rs`, `report.rs`, `graph.rs`, `dashboard.rs`, `check.rs`, `config.rs`) that all delegate the real work to `AnalysisService` / `Pipeline` in the library crate, so a single analysis pass (root detection → reachability → dynamic-ref detection → verdict evaluation → structural dead-code analysis) is computed once per invocation and reused across every section of a report.

---

## 🔬 Advanced / Model Training

Behind the `advanced` cargo feature, `ci` exposes additional hidden subcommands for building and calibrating your own models: `train`, `train-duplicate`, `calibrate`, `tune`, `export`, `merge`, `collect`, `export-feedback`, `update`, and `self-analyze` (the tool analyzing its own codebase). These are intended for maintainers iterating on the embedded ML models, not day-to-day usage — build with:

```bash
cargo install --path . --features advanced
```

---

## 🧪 Testing & Benchmarks

```bash
# Run unit and integration tests
cargo test --release

# Run property and fuzz tests
cargo test --test property_tests
cargo test --test fuzz_tests

# Run compression and graph benchmarks
cargo bench
```

---

## 📄 License

This project is licensed under the **MIT License**.
