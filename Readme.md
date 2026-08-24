```markdown
# Code Intelligence

[![Build Status](https://img.shields.io/github/actions/workflow/status/neontoshi/Code-intelligence/ci.yml?branch=main)](https://github.com/neontoshi/Code-intelligence/actions)
[![Test Coverage](https://img.shields.io/codecov/c/github/neontoshi/Code-intelligence)](https://codecov.io/gh/neontoshi/Code-intelligence)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![Model Accuracy](https://img.shields.io/badge/accuracy-95.3%25-brightgreen)](docs/evaluation_report.md)
[![Precision](https://img.shields.io/badge/precision-96.8%25-brightgreen)](docs/evaluation_report.md)

**High-precision semantic code intelligence engine for dead code detection, structural deduplication, and polyglot call-graph mapping.**

`code-intelligence` is a fast, multi-language static and dynamic analysis platform that combines AST parsing, graph reachability algorithms, and calibrated machine learning models to detect dead code with high precision, eliminate duplicate logic, and optimize large polyglot codebases[cite: 1, 2].

---

## ⚡ Key Highlights & Capabilities

* **Standalone Binary with Embedded ML**: Trained ML classifiers for dead code and duplicate detection are baked directly into the executable via compile-time embedding[cite: 2]. No external model files or runtime downloads required.
* **Unified Verdict Engine**: Combines static BFS reachability analysis, fan-in/fan-out graph metrics, dynamic reference detection, and temperature-calibrated linear ML models to classify symbols into 5 confidence states (`DefinitelyAlive`, `ProbablyAlive`, `Unknown`, `ProbablyDead`, `DefinitelyDead`)[cite: 1, 2].
* **Native Multi-Language Support (10 Languages)**:
  * **Rust**: `impl` blocks, traits, operator overloads, FFI, and macros[cite: 2].
  * **TypeScript / TSX / JavaScript**: ES6 modules, barrel exports, React components, hooks (`use*`), and event handlers[cite: 1, 2].
  * **Python**: Decorators (FastAPI, Flask, Pytest, Celery), magic dunder methods, `self.`/`cls.` calls, and `getattr()` string dispatches[cite: 1, 2].
  * **Go**: Export capitalization, receiver methods (`func (r *Repo)`), `init()` hooks, `Test*`/`Benchmark*` suites, and reflection[cite: 1, 2].
  * **Java**: Access modifiers, class methods, records, and Spring/Jakarta annotations (`@GetMapping`, `@Service`, `@Repository`)[cite: 1, 2].
  * **C# / .NET**: ASP.NET Core controllers, route attributes (`[HttpGet]`, `[HttpPost]`), MediatR handlers, and `Program.cs` entry points[cite: 2].
  * **Dart / Flutter**: Widget lifecycle methods (`build`, `initState`, `dispose`), state handlers, and `lib/main.dart` entry points[cite: 2].
  * **PHP**: Magic methods, Laravel/Symfony controller attributes, and dynamic execution (`call_user_func`)[cite: 2].
  * **C++**: Destructors, special member functions, entry macros, and header file declarations[cite: 2].
* **Dynamic Reference Detection**: Tracks reflection calls, dynamic imports (`import()`, `require()`), IPC bridges (Tauri `invoke(...)`, Electron `ipcRenderer.send(...)`), and string dispatch routes[cite: 1, 2].
* **Structural Duplicate Elimination**: Identifies duplicate blocks and code clones using MinHash, AST hashing, and ML pair classification to estimate token savings and suggest refactoring targets[cite: 1, 2].
* **Outcome Management**: Built-in tracking ledger (`.code-intelligence-outcomes.json`) records removals and false-positive dismissals to continuously improve training datasets[cite: 1, 2].
* **Interactive Terminal Dashboard**: Full-screen TUI built with Ratatui and Crossterm for live inspection, graph metrics, file-by-file categorization, and decision management[cite: 1, 2].
* **Visual Graph Output**: Exports interactive D3.js call graphs and circular architectural layer overviews in standalone HTML[cite: 1, 2].
* **Optional LLM Extensions**: Pluggable provider support (Ollama, OpenAI, Anthropic) for documentation generation, automated function summarization, and issue auditing[cite: 1, 2].

---

## 📦 Installation

### Quick Install (Pre-built Binaries)

#### **Linux & macOS**
```bash
curl -fsSL [https://raw.githubusercontent.com/neontoshi/Code-intelligence/main/install.sh](https://raw.githubusercontent.com/neontoshi/Code-intelligence/main/install.sh) | bash

```

#### **Windows (PowerShell as Administrator)**

```powershell
irm [https://raw.githubusercontent.com/neontoshi/Code-intelligence/main/install.ps1](https://raw.githubusercontent.com/neontoshi/Code-intelligence/main/install.ps1) | iex

```

#### **Windows (Command Prompt / Batch)**

Download and run [`install.bat`](https://www.google.com/search?q=https://raw.githubusercontent.com/neontoshi/Code-intelligence/main/install.bat) as Administrator.

---

### Build from Source (Cargo)

**Prerequisites:**

* [Rust & Cargo](https://rustup.rs/) (1.70+)



```bash
git clone [https://github.com/neontoshi/Code-intelligence.git](https://github.com/neontoshi/Code-intelligence.git)
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

Set default decision thresholds and optional LLM preferences in `~/.config/code-intelligence/config.toml`:

```bash
# Set decision threshold (default: 0.92)
ci config set threshold 0.92

# Configure optional LLM provider
ci config set llm_provider ollama
ci config set llm_model phi:2.7b

```

### 2. Scan a Codebase

Run dead code analysis on any target project:

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
| --- | --- | --- |
| `ci analyze [path]` | Scan project for dead functions, types, and modules

 | `ci analyze . --threshold 0.92 --git`<br> |
| `ci dedup [path]` | Find identical and structural code duplicates

 | `ci dedup . --threshold 0.85 --ml`<br> |
| `ci dashboard [path]` | Launch interactive terminal UI (Ratatui)

 | `ci dashboard .`<br> |
| `ci graph [path]` | Generate HTML call graphs (`interactive` or `overview`)

 | `ci graph . --output graph.html`<br> |
| `ci llm [path]` | Run deep semantic review and bug scan via LLM

 | `ci llm . --provider openai --model gpt-4`<br> |

### 2. Outcome Tracking & Management

| Command | Description | Example |
| --- | --- | --- |
| `ci list [path]` | List detected dead code candidates

 | `ci list --all`<br> |
| `ci remove <name>` | Mark dead candidate as deleted in the repo

 | `ci remove processOrder --commit 8f3d1b`<br> |
| `ci keep <name> "<reason>"` | Mark candidate as false positive / intentionally kept

 | `ci keep handlePing "Health check callback"`<br> |
| `ci stats [path]` | View removal rates and false-positive metrics

 | `ci stats --detailed`<br> |
| `ci report [path]` | Export markdown, JSON, or HTML analysis summaries

 | `ci report --format markdown --output report.md`<br> |

### 3. ML Training, Calibration & Data

| Command | Description | Example |
| --- | --- | --- |
| `ci train` | Train a linear classifier for dead code detection

 | `ci train --train-data data/train.json`<br> |
| `ci train-duplicate` | Train classifier for duplicate code detection

 | `ci train-duplicate data/pairs.json`<br> |
| `ci calibrate` | Calibrate confidence scores (temperature scaling)

 | `ci calibrate --method temperature --val-data data/val.json`<br> |
| `ci compare` | Compare accuracy and F1 across model configurations

 | `ci compare --train-data data/train.json`<br> |
| `ci features` | Display top differentiating features per language

 | `ci features --data combined_training.json`<br> |
| `ci ablation` | Run feature ablation studies to measure feature importance

 | `ci ablation --output-dir ./ablation_results`<br> |
| `ci export [path]` | Extract AST and graph feature vectors into training JSON

 | `ci export . --output features.json`<br> |
| `ci merge` | Deduplicate and split repo datasets into train/val/test splits

 | `ci merge --input "training_data/*.json" --dedup`<br> |

---

## 🖥️ Terminal Dashboard Navigation

Launch the full-screen terminal interface:

```bash
ci dashboard .

```

* **Summary Tab**: High-level project metrics, dead function percentage, and estimated removable lines of code.


* **Charts Tab**: Visual distribution of dead code across modules, languages, and confidence intervals.


* **List Tab**: Interactive table of candidate functions with detail inspection and evidence breakdown.


* **By File Tab**: File-by-file grouped breakdown of dead functions and types.


* **Priority Tab**: Ordered step-by-step removal plan minimizing breakage risk.


* **History Tab**: Audit log of confirmed removals and false-positive dismissals.



**Keybindings**:

* `Tab` / `Right` / `l`: Next tab


* `BackTab` / `Left` / `h`: Previous tab


* `Down` / `j` & `Up` / `k`: Navigate list items


* `g` / `G`: Jump to top / bottom


* `d`: Mark candidate as confirmed Dead


* `f`: Mark candidate as False Positive (prompts for reason)


* `s`: Defer candidate review


* `q` / `Esc`: Exit dashboard



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
│   ├── engine/               # Parser coordinator, index builder, caching, and pipeline stages
│   ├── graph/                # Call, dependency, import, and type graphs (Petgraph)
│   ├── llm/                  # Providers: Ollama, OpenAI, Anthropic, Mock
│   ├── ml/                   # Linear classifier, feature schema, temperature calibration
│   ├── optimize/             # Deduplication, MinHash, token estimation, compression
│   ├── output/               # Markdown, JSON, RAG generators, interactive HTML visualizations
│   └── parser/               # Tree-sitter multi-language AST extraction
├── models/                   # Pretrained and calibrated binary models (embedded into build)
└── data/                     # Training, validation, and test datasets

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

```

```
