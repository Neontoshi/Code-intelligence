```markdown
# Code Intelligence

[![Build Status](https://img.shields.io/github/actions/workflow/status/neontoshi/Code-intelligence/ci.yml?branch=main)](https://github.com/neontoshi/Code-intelligence/actions)
[![Test Coverage](https://img.shields.io/codecov/c/github/neontoshi/Code-intelligence)](https://codecov.io/gh/neontoshi/Code-intelligence)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![Model Accuracy](https://img.shields.io/badge/accuracy-95.3%25-brightgreen)](docs/evaluation_report.md)
[![Precision](https://img.shields.io/badge/precision-96.8%25-brightgreen)](docs/evaluation_report.md)

**Semantic Code Intelligence Engine for AI Dead Code Detection, Duplicate Detection, and Codebase Analysis**

`code-intelligence` is a fast, multi-language semantic analysis platform designed to map call graphs, detect dead code with high precision, eliminate structural duplication, and streamline refactoring across large polyglot codebases[cite: 1, 2].

---

## ⚡ Core Capabilities & Highlights

* **Unified Verdict Engine**: Combines static reachability analysis, fan-in/fan-out graph metrics, dynamic reference detection, and calibrated linear ML models to categorize symbols into `DefinitelyAlive`, `ProbablyAlive`, `Unknown`, `ProbablyDead`, or `DefinitelyDead`[cite: 1, 2].
* **Polyglot AST Support**: Native Tree-Sitter parsing and resolution across **10 languages**:
  * **Rust**: `impl` blocks, traits, operator overloads, FFI, and macros[cite: 2].
  * **TypeScript / TSX / JavaScript**: ES6 modules, barrel exports, React component lifecycle methods, hooks (`use*`), and UI event handlers[cite: 1, 2].
  * **Python**: Decorators (FastAPI, Flask, Pytest, Celery), dunder magic methods, `self.`/`cls.` invocations, and `getattr()` string dispatches[cite: 1, 2].
  * **Go**: Export capitalization, receiver methods (`func (r *Repo)`), `init()` hooks, `Test*`/`Benchmark*` suites, and `reflect.MethodByName`[cite: 1, 2].
  * **Java**: Access modifiers, class methods (`this.`), record types, and Spring/Jakarta annotations (`@GetMapping`, `@Service`, `@Repository`)[cite: 1, 2].
  * **C# / .NET**: ASP.NET Core controllers, routes (`[HttpGet]`, `[HttpPost]`), MediatR handlers, and `Program.cs` / `Startup.cs` entry points[cite: 2].
  * **Dart / Flutter**: Widget lifecycle methods (`build`, `initState`, `dispose`), state handlers, and `lib/main.dart` application roots[cite: 2].
  * **PHP**: Magic methods, Laravel/Symfony controller attributes, and dynamic execution (`call_user_func`)[cite: 2].
  * **C++**: Destructors, special member functions, entry macros, and header file declarations[cite: 2].
* **Smart Dynamic Reference Detection**: AST pattern extractors track reflection, dynamic imports (`import()`, `require()`), IPC bridges (Tauri `invoke(...)`, Electron `ipcRenderer.send(...)`), and string-based routing dispatches[cite: 1, 2].
* **Structural Duplicate Elimination**: Identifies duplicate blocks and clones using MinHash, AST hashing, and ML-based duplicate classification to calculate token savings and refactoring suggestions[cite: 1, 2].
* **Outcome Management**: Built-in tracking ledger (`.code-intelligence-outcomes.json`) records removals and false-positive dismissals to continuously fine-tune training datasets[cite: 1, 2].
* **Interactive Terminal Dashboard**: Full-featured TUI built with Ratatui and Crossterm for live inspection, graph metrics, file-by-file categorization, and decision management[cite: 1, 2].
* **Visual Graph Output**: Exports interactive D3.js call graphs and circular architectural layer overviews in standalone HTML[cite: 1, 2].
* **Optional LLM Integration**: Pluggable provider support (Ollama, OpenAI, Anthropic) for documentation generation, automated function summarization, and issue auditing[cite: 1, 2].

---

## 📦 Installation

### Prerequisites

* [Rust & Cargo](https://rustup.rs/) (1.70+)[cite: 1, 2]
* (Optional) [Ollama](https://ollama.com/) running locally for offline LLM features[cite: 1, 2]

### Build & Install

```bash
git clone [https://github.com/neontoshi/Code-intelligence.git](https://github.com/neontoshi/Code-intelligence.git)
cd Code-intelligence
cargo install --path .

```

Verify the installation:

```bash
ci --version
ci --help

```

---

## 🚀 Quick Start

### 1. Global Configuration

Set up default model paths, classification thresholds, and LLM preferences in `~/.config/code-intelligence/config.toml`:

```bash
# Set default calibrated classification model
ci config set model models/dead_code_model_v4_balanced_calibrated.bin

# Set decision threshold (default: 0.92)
ci config set threshold 0.92

# Configure LLM provider (optional)
ci config set llm_provider ollama
ci config set llm_model phi:2.7b

```

### 2. Run Dead Code Check

Scan a project directory to generate a full dead code report:

```bash
ci analyze ~/Documents/my-project

```

### 3. Launch Interactive Terminal Dashboard

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

 | `ci dedup . --threshold 0.85`<br> |
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

### 3. ML Training, Calibration & Experimentation

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

### 4. Training Data Utilities

| Command | Description | Example |
| --- | --- | --- |
| `ci export [path]` | Extract AST and graph feature vectors into training JSON

 | `ci export . --output features.json`<br> |
| `ci merge` | Deduplicate and split repo datasets into train/val/test splits

 | `ci merge --input "training_data/*.json" --dedup`<br> |
| `ci collect` | Clone public repositories and generate bulk training sets

 | `ci collect --max-repos 25`<br> |
| `ci self-analyze` | Run full analysis pipeline on `code-intelligence` itself

 | `ci self-analyze --format full`<br> |

---

## 🧰 Standalone Cargo Binaries

You can also run specialized tools directly using Cargo:

```bash
# Core analyzers
cargo run --release --bin dead_code_check -- ./path/to/project --threshold 0.92
cargo run --release --bin dedup_check -- ./path/to/project --threshold 0.85
cargo run --release --bin dead_code_dashboard -- ./path/to/project

# ML Pipeline & Calibration
cargo run --release --bin data
cargo run --release --bin train -- --train-data data/train.json
cargo run --release --bin evaluate -- detailed --model model.bin --test-data data/test.json

```

---

## 🖥️ Terminal Dashboard Navigation

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
├── models/                   # Pretrained and calibrated binary models
└── data/                     # Training, validation, and test datasets

```

---

## 📄 License

This project is licensed under the **MIT License**.

```

```
