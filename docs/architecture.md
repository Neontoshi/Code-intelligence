# Architecture Overview

## High-Level Architecture

```
┌───────────────────────────────────────────────────────────────────────┐
│                          CODE-INTELLIGENCE                             │
│                                                                         │
│  ┌────────────────┐   ┌────────────────┐   ┌────────────────────┐     │
│  │   CLI Layer     │   │  Dashboard UI   │   │     API Layer      │     │
│  │   (ci binary)   │   │  (ratatui TUI)  │   │    (Public API)    │     │
│  └────────┬────────┘   └────────┬────────┘   └──────────┬─────────┘     │
│           │                     │                        │             │
│           └─────────────────────┼────────────────────────┘             │
│                                 │                                      │
│                                 ▼                                      │
│  ┌───────────────────────────────────────────────────────────────┐    │
│  │                        PIPELINE ENGINE                        │    │
│  │                                                                │    │
│  │   ┌─────────┐   ┌─────────┐   ┌──────────┐   ┌──────────┐     │    │
│  │   │ Collect │ → │  Parse  │ → │ Analyze  │ → │ Optimize │     │    │
│  │   │  Files  │   │   AST   │   │  Graphs  │   │ Features │     │    │
│  │   └─────────┘   └─────────┘   └──────────┘   └──────────┘     │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                                 │                                      │
│                                 ▼                                      │
│  ┌───────────────────────────────────────────────────────────────┐    │
│  │                        ANALYSIS LAYER                         │    │
│  │                                                                │    │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐             │    │
│  │  │ Dead Code  │   │ Duplicate  │   │  Explain-  │             │    │
│  │  │ Detection  │   │ Detection  │   │  ability   │             │    │
│  │  └────────────┘   └────────────┘   └────────────┘             │    │
│  │                                                                │    │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐             │    │
│  │  │  Verdict   │   │  Feature   │   │  Outcome   │             │    │
│  │  │  Engine    │   │ Extraction │   │  Tracking  │             │    │
│  │  └────────────┘   └────────────┘   └────────────┘             │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                                 │                                      │
│                                 ▼                                      │
│  ┌───────────────────────────────────────────────────────────────┐    │
│  │                          ML LAYER                              │    │
│  │                                                                │    │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐             │    │
│  │  │ Classifier │   │Calibration │   │ Duplicate  │             │    │
│  │  │ (Logistic  │   │(Temperature│   │ Classifier │             │    │
│  │  │Regression) │   │  Scaling)  │   │            │             │    │
│  │  └────────────┘   └────────────┘   └────────────┘             │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────┐    │
│  │                          LLM LAYER                             │    │
│  │                                                                │    │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐             │    │
│  │  │  Ollama    │   │   OpenAI   │   │ Anthropic  │             │    │
│  │  │  Provider  │   │  Provider  │   │  Provider  │             │    │
│  │  └────────────┘   └────────────┘   └────────────┘             │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────┐    │
│  │                        PARSER LAYER                            │    │
│  │                                                                │    │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐             │    │
│  │  │Tree-sitter │   │  Semantic  │   │  Comment   │             │    │
│  │  │  Parser    │   │  Analyzer  │   │  Analyzer  │             │    │
│  │  └────────────┘   └────────────┘   └────────────┘             │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────┐    │
│  │                        GRAPH LAYER                             │    │
│  │                                                                │    │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐             │    │
│  │  │Call Graph  │   │Type Graph  │   │  Import    │             │    │
│  │  │            │   │            │   │  Graph     │             │    │
│  │  └────────────┘   └────────────┘   └────────────┘             │    │
│  │                                                                │    │
│  │  ┌────────────┐   ┌────────────┐                              │    │
│  │  │ Dependency │   │  Project   │                              │    │
│  │  │  Graph     │   │  Graph     │                              │    │
│  │  └────────────┘   └────────────┘                              │    │
│  └───────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  ┌───────────────────────────────────────────────────────────────┐    │
│  │                        OUTPUT LAYER                            │    │
│  │                                                                │    │
│  │  ┌────────────┐   ┌────────────┐   ┌────────────┐             │    │
│  │  │    JSON    │   │  Markdown  │   │    HTML    │             │    │
│  │  │   Output   │   │   Output   │   │   Graphs   │             │    │
│  │  └────────────┘   └────────────┘   └────────────┘             │    │
│  └───────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
code-intelligence/
├── src/
│   ├── analysis/          # Analysis logic
│   │   ├── dead_code/     # Dead code detection
│   │   ├── verdict/       # Verdict engine
│   │   ├── features.rs    # Feature extraction
│   │   ├── roots.rs       # Root detection
│   │   └── explainability.rs  # Explanations
│   │
│   ├── bin/               # CLI tools
│   │   ├── ci.rs          # Main CLI
│   │   ├── dead_code_check.rs
│   │   ├── dead_code_dashboard.rs
│   │   └── common/        # Shared utilities
│   │
│   ├── engine/            # Pipeline engine
│   │   ├── pipeline.rs    # Main pipeline
│   │   ├── cache.rs       # Caching
│   │   ├── incremental.rs # Incremental analysis
│   │   └── config.rs      # Configuration
│   │
│   ├── graph/             # Graph representations
│   │   ├── call_graph.rs
│   │   ├── type_graph.rs
│   │   ├── import_graph.rs
│   │   └── project_graph.rs
│   │
│   ├── ml/                # Machine learning
│   │   ├── classifier.rs
│   │   ├── calibration.rs
│   │   ├── duplicate_classifier.rs
│   │   └── feature_schema.rs
│   │
│   ├── llm/                # LLM integration
│   │   ├── providers/
│   │   │   ├── ollama.rs
│   │   │   ├── openai.rs
│   │   │   └── anthropic.rs
│   │   └── code_understanding.rs
│   │
│   ├── parser/             # Parsing
│   │   └── tree_sitter.rs  # Tree-sitter integration
│   │
│   ├── optimize/           # Optimization
│   │   └── dedup/          # Duplicate detection
│   │
│   ├── output/              # Output generation
│   │   ├── json.rs
│   │   ├── markdown.rs
│   │   ├── interactive.rs
│   │   └── overview.rs
│   │
│   └── utils/                # Utilities
│       ├── parallel.rs
│       ├── hashing.rs
│       └── safe.rs
│
├── tests/                 # Tests
├── models/                # ML models
├── data/                  # Training data
└── docs/                  # Documentation
```

---

## Core Components

### 1. Pipeline Engine

The pipeline is the heart of the system. It processes a project through the following stages:

```rust
pub struct Pipeline {
    parser: TreeSitterParser,
    scorer: ImportanceScorer,
    cache: FileCache,
    config: PipelineConfig,
    llm_provider: Option<Arc<dyn LLMProvider>>,
    analysis_cache: Option<AnalysisCacheManager>,
    file_tracker: Option<FileTracker>,
    enable_incremental: bool,
}
```

**Stages:**

| Stage | Input | Output | Description |
|-------|-------|--------|-------------|
| **Collect** | Path | RawProject | Find all source files |
| **Parse** | RawProject | ParsedProject | Parse AST with tree-sitter |
| **Analyze** | ParsedProject | AnalyzedProject | Build graphs |
| **Optimize** | AnalyzedProject | OptimizedProject | Extract features, build indexes |
| **Finalize** | OptimizedProject | ProjectAnalysis | Full analysis result |

### 2. Graph System

Multiple graphs represent different relationships:

#### Call Graph
- Nodes: functions
- Edges: calls between functions
- Used for: reachability, fan-in/fan-out

#### Type Graph
- Nodes: types (structs, enums, traits, classes)
- Edges: inheritance, implementation
- Used for: trait detection, polymorphism

#### Import Graph
- Nodes: modules/files
- Edges: import relationships
- Used for: unused import detection, module deadness

#### Project Graph
- Unified graph combining all relationships
- Used for: cross-cutting analysis

### 3. Dead Code Detection Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    FUNCTION CANDIDATE                            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   1. is_never_dead()                              │
│  Checks whether a function should never be considered dead:      │
│  - Trait implementations                                         │
│  - React components/hooks                                        │
│  - Framework decorators                                          │
│  - Entry points                                                  │
│  - FFI exports                                                   │
│  - Test functions                                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   2. Root Detection                               │
│  Finds all entry points:                                         │
│  - main(), run(), start()                                        │
│  - Public API functions                                          │
│  - Test functions                                                │
│  - Framework callbacks                                           │
│  - FFI exports                                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   3. Reachability Analysis                        │
│  BFS from roots through the call graph:                          │
│  - Reachable → Alive                                             │
│  - Unreachable → candidate for dead                               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   4. Feature Extraction                           │
│  Extracts 46 features:                                           │
│  - Graph: fan_in, fan_out, depth, cycle                          │
│  - Signature: params, returns, public, async                     │
│  - Name: patterns, length                                        │
│  - File: test, benches, generated                                 │
│  - Type: method, trait_impl, associated                          │
│  - Complexity: cyclomatic                                        │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   5. ML Prediction                                 │
│  Logistic regression:                                            │
│  - Outputs a probability of being dead                           │
│  - Threshold: 0.80 (recommended)                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                   6. Verdict Engine                                │
│  Combines the static and ML signals:                             │
│  final_score = 0.6 * static + 0.4 * ml                           │
│  Evaluated top-down, first match wins:                           │
│  - ≥ 0.85          → Definitely Dead                              │
│  - ≥ 0.70          → Probably Dead                                │
│  - ≥ 0.30          → Unknown                                     │
│  - ≥ 0.15          → Probably Alive                               │
│  - < 0.15          → Definitely Alive                              │
└─────────────────────────────────────────────────────────────────┘
```

> **Note:** step 5's "Threshold: 0.80" and step 6's "≥ 0.85 → Definitely Dead" are two different numbers on what looks like two different scores — the raw ML probability vs. the fused `final_score`. The doc doesn't say how (or whether) the 0.80 ML threshold feeds into the verdict thresholds below it; worth clarifying which one actually gates the CLI's dead/alive output.

### 4. ML System

#### Feature Schema (46 Features)

| Category | Count | Features |
|----------|-------|----------|
| **Graph** | 4 | fan_in, fan_out, call_depth, is_cycle |
| **Signature** | 4 | param_count, return_count, is_public, is_async |
| **Complexity** | 1 | complexity |
| **Name** | 26 | contains patterns, starts/ends, length |
| **File** | 5 | test, benches, meta, examples, generated |
| **Type** | 6 | is_method, trait_impl, associated, names |

#### Model Training

```rust
// Training pipeline
1. Load training examples
2. Extract features
3. Train logistic regression
4. Calibrate with temperature scaling
5. Evaluate on test set
6. Save model
```

### 5. LLM Integration

#### Providers

| Provider | Model | Use Case |
|----------|-------|----------|
| **Ollama** | phi:2.7b | Local, offline |
| **OpenAI** | gpt-3.5/4 | Cloud, high quality |
| **Anthropic** | claude-3 | Cloud, high quality |
| **Mock** | – | Testing |

#### Capabilities

```rust
// LLM capabilities
- summarize_function()      // 1-2 sentence summary
- explain_function()        // Detailed explanation
- analyze_bugs()             // Bug detection
- suggest_improvements()    // Code improvements
- generate_documentation()  // Project docs
- analyze_architecture()    // Architecture analysis
- analyze_duplicate_pair()  // Duplicate verification
```

### 6. Cache System

#### Cache Layers

| Layer | Storage | Description |
|-------|---------|-------------|
| **Memory** | DashMap | In-memory cache |
| **Disk** | Files | Persistent cache |

#### Cache Keys

```rust
// Cache keys
- File hash (SHA256)   // File content
- Project hash          // Project structure
- Function hash         // Function signature
```

### 7. Incremental Analysis

```rust
// Incremental analysis flow
1. Detect changed files
2. Find affected functions
3. Re-analyze only what changed
4. Update cache
5. Return updated analysis
```

---

## Data Flow

### Analysis Flow

```
User Input (Path)
       │
       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Pipeline.process_project()                    │
└─────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Stage 1: File Collection                      │
│  - Walk directory                                                │
│  - Filter by extension                                           │
│  - Skip ignored directories                                      │
└─────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Stage 2: Parsing                               │
│  - Parse each file with tree-sitter                              │
│  - Extract functions, types, imports                             │
│  - Extract comments, decorators                                  │
└─────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Stage 3: Graph Building                        │
│  - Build call graph                                               │
│  - Build type graph                                               │
│  - Build import graph                                             │
│  - Build project graph                                            │
└─────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Stage 4: Analysis                               │
│  - Root detection                                                 │
│  - Reachability analysis                                          │
│  - Feature extraction                                             │
│  - ML prediction                                                  │
│  - Verdict generation                                             │
└─────────────────────────────────────────────────────────────────┘
       │
       ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Stage 5: Output                                 │
│  - Markdown report                                                │
│  - JSON report                                                    │
│  - HTML graphs                                                    │
│  - Dashboard UI                                                   │
└─────────────────────────────────────────────────────────────────┘
       │
       ▼
Output (Report, Dashboard, etc.)
```

---

## Performance Characteristics

### Memory Usage

| Component | Typical Size | Notes |
|-----------|--------------|-------|
| ParsedFile | 10–50 KB | Per file |
| FunctionNode | 1–2 KB | Per function |
| CallGraph (100k nodes) | 200–400 MB | Depends on edge count |
| Feature vectors | 1–2 KB | Per function |

### Time Complexity

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Parsing | O(N) | N = lines of code |
| Graph Building | O(F + C) | F = functions, C = calls |
| Reachability | O(F + C) | BFS |
| ML Prediction | O(F) | F = functions |
| Cycle Detection | O(F + C) | Skipped for > 5000 nodes |

### Scaling

| Size | Functions | Time | Memory |
|------|-----------|------|--------|
| Small | < 1,000 | < 5s | < 100 MB |
| Medium | 1k–10k | 5–30s | 100–500 MB |
| Large | 10k–50k | 30s–3min | 500 MB–1 GB |
| Huge | 50k+ | 3–10min | 1–4 GB |

---

## Security Considerations

### Code Analysis

- **No code execution**: all analysis is static
- **No network calls**: offline by default
- **File system access**: read-only, except for the cache

### LLM Integration

- **API keys**: stored in a local config file
- **Data sent**: only function signatures and source previews
- **Privacy**: no code is stored on LLM servers

### Caching

- **Cache location**: project directory (`.code-intelligence-cache`)
- **Cache invalidation**: content-hash based
- **Cache security**: same as project files

---

## Extensibility

### Adding a New Language

1. Add the tree-sitter grammar to `Cargo.toml`
2. Add a language config in `TreeSitterParser`
3. Add language-specific patterns
4. Test with fixtures

### Adding a New Feature

1. Define the feature in `feature_schema.rs`
2. Extract the feature in `features.rs`
3. Add it to the `FunctionFeatures` struct
4. Update model training

### Adding a New Output Format

1. Create a new module in `output/`
2. Implement a `generate()` function
3. Add it to `ci report --format`

### Adding a New LLM Provider

1. Create a provider in `llm/providers/`
2. Implement the `LLMProvider` trait
3. Add it to the provider factory
4. Test with the mock provider

---

## Testing Strategy

### Test Types

| Type | Location | Description |
|------|----------|-------------|
| **Unit** | `tests/unit/` | Individual components |
| **Integration** | `tests/integration/` | End-to-end flows |
| **Fuzz** | `tests/fuzz_tests.rs` | Random inputs |
| **Property** | `tests/property_tests.rs` | Invariant testing |
| **Golden** | `tests/integration/golden_test.rs` | Expected outputs |

### Test Coverage

| Module | Coverage |
|--------|----------|
| `parser/` | 85% |
| `graph/` | 80% |
| `analysis/` | 75% |
| `ml/` | 70% |
| `engine/` | 65% |
| `bin/` | 60% |

---

## Deployment Options

### Local Development

```bash
cargo install --path .
ci analyze .
```

### CI/CD Pipeline

```yaml
# GitHub Actions
jobs:
  dead-code:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install ci
        run: cargo install --path .
      - name: Run analysis
        run: ci analyze . --format json
```

> **Note:** the original snippet had `run: ci ci . --format json` — doubled binary name, and a different subcommand (`ci`) than the `analyze` used in Local Development above. Changed to match `ci analyze .`, but flagging in case the CI/CD example was actually meant to invoke a different subcommand.

### Docker

```dockerfile
FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build --release
ENTRYPOINT ["/app/target/release/ci"]
```

---

## Monitoring & Observability

### Logging

```rust
// Structured logging
logger.info("analysis_started", {
    "project": project_path,
    "files": file_count,
    "functions": function_count
});
```

### Metrics

```rust
// Performance metrics
- analysis_time_ms
- memory_usage_mb
- files_parsed
- functions_analyzed
- dead_functions_found
- cache_hit_rate
```

### Error Handling

```rust
// Error taxonomy
- ParseError: invalid syntax
- GraphError: cycle detected
- ModelError: invalid model
- MemoryLimitExceeded: out of memory
- Timeout: analysis took too long
```

---

## Future Architecture Plans

### Planned Improvements

1. **Distributed Analysis**: split analysis across multiple machines
2. **Real-time Analysis**: watch mode for instant feedback
3. **Web Interface**: full web-based dashboard
4. **Plugins**: extensible plugin system
5. **More Languages**: Swift, Kotlin
6. **Better ML**: deep learning models (transformers)
7. **Cross-Project Analysis**: detect dead code across repos
