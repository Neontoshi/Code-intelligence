# Code Intelligence

**Semantic Code Intelligence Engine for AI - Dead Code Detection, Duplicate Detection, and Code Analysis**

A comprehensive toolkit for analyzing, understanding, and optimizing codebases using ML-powered dead code detection, duplicate detection, call graph analysis, and LLM integration. Supports Rust, TypeScript, JavaScript, Python, Go, and Java.

---

## 🚀 Quick Start

```bash
# Install everything with one command
git clone https://github.com/yourusername/code-intelligence
cd code-intelligence
cargo install --path .

# First-time setup
ci config set model ~/Documents/code-intelligence/model_verified_v2.bin
ci config set threshold 0.55

# Analyze your project
cd ~/Documents/your-project
ci analyze

# View results
ci list
ci stats

# Remove dead code
ci remove publishGiveaway
```

---

## 📦 Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/code-intelligence
cd code-intelligence

# Install ALL binaries with one command
cargo install --path .

# This installs: ci, dead_code_check, train_model, update_outcome, dedup_check, and more
```

### Install Specific Binary

```bash
# Install only the main CLI
cargo install --path . --bin ci

# Install only the dead code checker
cargo install --path . --bin dead_code_check

# Install only the deduplication checker
cargo install --path . --bin dedup_check
```

### Verify Installation

```bash
# Check that the CLI is installed
which ci
# Should show: /home/username/.cargo/bin/ci

# Test it
ci --version
ci --help
```

---

## 🎯 What It Does

| Feature | Description |
|---------|-------------|
| **Dead Code Detection** | ML-powered detection of unused functions, types, and modules |
| **Duplicate Code Detection** | Find and refactor duplicate code across your codebase |
| **Call Graph Analysis** | Visualize function relationships and dependencies |
| **Import Graph** | Track module dependencies and imports |
| **Type Graph** | Understand type relationships and usage |
| **Dynamic Reference Detection** | Find reflection, callbacks, and framework-based references |
| **Reachability Analysis** | Determine which functions are reachable from entry points |
| **LLM Integration** | Summarize functions, find bugs, suggest improvements |
| **Git Analysis** | Track code age and activity |
| **Interactive Dashboard** | Terminal UI for reviewing dead code |
| **Outcome Tracking** | Track which dead functions were actually removed |

---

## 📋 Commands

### Core Analysis

| Command | Description | Example |
|---------|-------------|---------|
| `ci analyze` | Detect dead code in a project | `ci analyze ~/Documents/Kyma` |
| `ci dedup` | Find duplicate code | `ci dedup ~/Documents/X_giveaway_system` |
| `ci graph` | Generate call graph | `ci graph ~/Kyma --format png` |
| `ci llm` | LLM-powered analysis | `ci llm ~/Kyma --provider ollama` |
| `ci dashboard` | Interactive terminal UI | `ci dashboard ~/Documents/Kyma` |

### Outcome Management

| Command | Description | Example |
|---------|-------------|---------|
| `ci list` | List dead functions found | `ci list` |
| `ci remove` | Mark a function as removed | `ci remove publishGiveaway` |
| `ci keep` | Mark a function as false positive | `ci keep uploadImage "Used in tests"` |
| `ci stats` | Show outcome statistics | `ci stats --detailed` |
| `ci report` | Generate a report | `ci report --format html` |

### Training & Model Management

| Command | Description | Example |
|---------|-------------|---------|
| `ci train` | Train the ML model | `ci train --precision 0.95` |
| `ci calibrate` | Calibrate model confidence | `ci calibrate --method temperature` |
| `ci tune` | Tune confidence threshold | `ci tune --precision 0.99` |
| `ci compare` | Compare ML models | `ci compare` |
| `ci features` | Analyze feature importance | `ci features` |
| `ci evaluate` | Evaluate model per language | `ci evaluate --detailed` |

### Data Management

| Command | Description | Example |
|---------|-------------|---------|
| `ci export` | Export training data from a project | `ci export ~/Kyma --output training.json` |
| `ci merge` | Merge training data files | `ci merge --dedup` |
| `ci self-analyze` | Analyze code-intelligence itself | `ci self-analyze --format full` |

### Configuration

| Command | Description | Example |
|---------|-------------|---------|
| `ci config set` | Set a config value | `ci config set threshold 0.55` |
| `ci config get` | Get a config value | `ci config get model` |
| `ci config list` | List all config values | `ci config list` |

---

## 🔧 Configuration

### First-Time Setup

```bash
# Set the ML model path (required for analysis)
ci config set model ~/Documents/code-intelligence/model_verified_v2.bin

# Set default confidence threshold (0.0 - 1.0)
ci config set threshold 0.55

# Enable verbose output
ci config set verbose true

# Configure LLM (optional)
ci config set llm_provider ollama
ci config set llm_model phi:2.7b
```

### Configuration File

Global config is stored at:
```
~/.config/code-intelligence/config.toml
```

Example:
```toml
[defaults]
model = "/home/user/code-intelligence/model_verified_v2.bin"
threshold = 0.55
verbose = false
llm_provider = "ollama"
llm_model = "phi:2.7b"

[projects."/home/user/Documents/X_giveaway_system"]
type = "typescript"
threshold = 0.55
last_analyzed = "2026-08-09"
dead_count = 20

[projects."/home/user/Documents/Kyma"]
type = "mixed"
threshold = 0.40
last_analyzed = "2026-08-09"
dead_count = 54
```

---

## 📊 Workflow Examples

### 1. Complete Project Analysis

```bash
# Navigate to your project
cd ~/Documents/my-project

# First analysis
ci analyze

# Review results
ci list

# Check statistics
ci stats

# Remove dead functions you've deleted
ci remove unused_function_1
ci remove unused_function_2

# Mark false positives
ci keep helper_function "Used in tests"

# Generate a report
ci report --format markdown --output report.md

# Re-analyze to confirm clean
ci analyze
ci stats
```

### 2. Find and Refactor Duplicate Code

```bash
cd ~/Documents/Kyma

# Find duplicates with similarity threshold 0.85
ci dedup --threshold 0.85

# Use ML model for better detection
ci dedup --ml
```

### 3. Generate Call Graph Visualization

```bash
cd ~/Documents/Kyma

# Generate full call graph
ci graph --format dot

# Generate focused graph for a specific function
ci graph --entry "main" --depth 3

# Generate as PNG (requires graphviz)
ci graph --format png
```

### 4. LLM-Powered Analysis

```bash
cd ~/Documents/Kyma

# Run LLM analysis with Ollama
ci llm --provider ollama

# Use OpenAI GPT-4
ci llm --provider openai --model gpt-4 --api-key $OPENAI_API_KEY

# Custom temperature and max tokens
ci llm --temperature 0.1 --max-tokens 2000
```

### 5. Train a Custom Model

```bash
# Export training data from your projects
ci export ~/Documents/Kyma --output training_data/kyma.json
ci export ~/Documents/X_giveaway_system --output training_data/x_giveaway.json

# Merge all training data
ci merge --dedup

# Train the model
ci train --precision 0.95

# Calibrate the model
ci calibrate --method temperature

# Tune the threshold
ci tune --precision 0.99

# Test the new model
ci analyze --threshold 0.55
```

### 6. Interactive Dashboard

```bash
cd ~/Documents/Kyma

# Open the dashboard
ci dashboard

# Navigate with arrow keys
# Press 'q' to quit
```

---

## 🎨 Output Formats

| Format | Description | Use Case |
|--------|-------------|----------|
| **Markdown** | Human-readable report | Documentation, PR descriptions |
| **JSON** | Machine-readable data | CI/CD, API integration |
| **Full** | Comprehensive analysis | Deep dives, code reviews |
| **Graphviz (DOT)** | Call graph visualization | Visualizing dependencies |
| **HTML** | Interactive report | Sharing with team |

---

## 🧠 Supported Languages

| Language | Support Level |
|----------|---------------|
| **Rust** | ✅ Full (traits, impls, macros) |
| **TypeScript** | ✅ Full (React, NestJS, decorators) |
| **JavaScript** | ✅ Full (JSX, React, Node.js) |
| **Python** | ✅ Full (decorators, Flask, FastAPI) |
| **Go** | ✅ Full (interfaces, exports) |
| **Java** | ✅ Full (annotations, Spring) |

---

## 🛠️ All Binaries

Running `cargo install --path .` installs all these tools:

| Binary | Purpose |
|--------|---------|
| `ci` | Main CLI - All-in-one command tool |
| `dead_code_check` | Core dead code analyzer |
| `dedup_check` | Find duplicate code |
| `train_model` | Train ML model |
| `calibrate_model` | Calibrate model confidence |
| `tune_threshold` | Find optimal threshold |
| `update_outcome` | Track removal outcomes |
| `verify_dead_candidates` | Generate review checklist |
| `training_data_exporter` | Export training data |
| `merge_all_training_data` | Merge training datasets |
| `train_duplicate_model` | Train duplicate detection model |
| `analyze_features_per_language` | Feature importance analysis |
| `evaluate_per_language` | Evaluate model per language |
| `feature_ablation` | Determine which features matter |
| `model_comparison` | Compare ML algorithms |
| `dead_code_dashboard` | Terminal UI dashboard |

---

## 📁 File Locations

| File | Purpose |
|------|---------|
| `~/.config/code-intelligence/config.toml` | Global configuration |
| `./.code-intelligence-outcomes.json` | Per-project outcomes tracking |
| `model.bin` | Trained ML model |
| `model_calibrated.bin` | Calibrated ML model |
| `data/train.json` | Training data |
| `data/val.json` | Validation data |
| `data/test.json` | Test data |
| `training_data/*.json` | Raw training data per repository |
| `call_graph.dot` | Call graph in DOT format |

---

## 🔍 Troubleshooting

### "Command not found: ci"

```bash
# Reinstall
cargo install --path .

# Or add to PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

### "No model configured"

```bash
ci config set model ~/Documents/code-intelligence/model_verified_v2.bin
```

### "No tracked outcomes found"

```bash
# Run analysis first
ci analyze
```

### "No pending function found matching 'x'"

```bash
# Check the exact function name
ci list
```

### "dead_code_check not found"

```bash
# Install all binaries
cargo install --path .
```

### Stack overflow on large projects

```bash
# Use a lower threshold for large projects
ci analyze --threshold 0.40
```

---

## 🔄 CI/CD Integration

### GitHub Actions

```yaml
name: Dead Code Check
on: [push, pull_request]

jobs:
  dead-code:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install Code Intelligence
        run: |
          git clone https://github.com/yourusername/code-intelligence
          cd code-intelligence
          cargo install --path .
      - name: Check Dead Code
        run: |
          ci analyze
          ci stats
          ci report --format json --output report.json
      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: dead-code-report
          path: report.json
```

### Git Pre-Commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
if ci stats 2>/dev/null | grep -q "Pending: [1-9]"; then
    echo "⚠️ There are pending dead code findings."
    echo "   Run 'ci list' to see them."
    echo "   Run 'ci remove <name>' after deleting them."
    echo "   Run 'ci keep <name> \"reason\"' if they're false positives."
    exit 1
fi
```

---

## 📈 Performance

| Metric | Value |
|--------|-------|
| **Precision** | Up to 100% (at threshold 0.55) |
| **Recall** | 5-60% depending on threshold |
| **Speed** | ~0.5-3s for 500 functions |
| **Languages** | 6+ languages supported |
| **Model Size** | ~3KB |

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific tests
cargo test --lib -- --nocapture

# Test on code-intelligence itself
ci self-analyze
```

---

## 📝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

---

## 📄 License

MIT License

---

## 🙏 Acknowledgments

- [tree-sitter](https://tree-sitter.github.io/tree-sitter/) - Language parsing
- [petgraph](https://github.com/petgraph/petgraph) - Graph algorithms
- [linfa](https://github.com/rust-ml/linfa) - ML framework
- [Ollama](https://ollama.ai/) - Local LLM support

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/code-intelligence/issues)
- **Documentation**: [Wiki](https://github.com/yourusername/code-intelligence/wiki)
- **Discord**: [Join our Discord](https://discord.gg/your-invite)

---

**Built with ❤️ by the Code Intelligence Team**
