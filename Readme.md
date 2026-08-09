# Code Intelligence CLI (`ci`)

A global CLI tool for detecting and managing dead code across any project. Built with ML-powered analysis for 10+ languages.

## Quick Start

```bash
# Install globally
cargo install --path . --bin ci

# First-time setup
ci config set model ~/Documents/code-intelligence/model_verified_v2.bin
ci config set threshold 0.55

# Analyze current project
cd ~/Documents/X_giveaway_system
ci analyze

# List dead functions found
ci list

# Mark a function as removed
ci remove publishGiveaway

# Check statistics
ci stats
```

## Installation

### From Source

```bash
git clone https://github.com/yourusername/code-intelligence
cd code-intelligence
cargo build --release
cargo install --path . --bin ci
```

### Verify Installation

```bash
ci --version
ci --help
```

## Configuration

### First-Time Setup

Configure your global settings once:

```bash
# Set the ML model path
ci config set model ~/Documents/code-intelligence/model_verified_v2.bin

# Set default confidence threshold (0.0 - 1.0)
ci config set threshold 0.55

# Enable verbose output
ci config set verbose true
```

### View Configuration

```bash
# List all config values
ci config list

# Get a specific value
ci config get model
ci config get threshold
```

## Commands

### `ci analyze [path]`

Analyze a project for dead code.

```bash
# Analyze current directory
ci analyze

# Analyze specific project
ci analyze ~/Documents/Kyma

# Analyze with custom threshold
ci analyze --threshold 0.40

# Verbose output
ci analyze --verbose
```

**What it does:**
- Detects project type (Rust, TypeScript, Python, Go, Java, etc.)
- Runs ML-based dead code analysis
- Tracks results in `.code-intelligence-outcomes.json`
- Updates project configuration

---

### `ci list [path]`

List all pending dead functions found in a project.

```bash
# List in current directory
ci list

# List in specific project
ci list ~/Documents/Kyma
```

**Output:**
```
📋 Pending Dead Functions (20 total):
   (Use `ci remove <name>` or `ci keep <name> "reason"`)

| # | Function | Confidence | File |
|---|----------|------------|------|
| 1 | publishGiveaway | 82.7% | api.ts |
| 2 | updateGiveaway | 82.1% | api.ts |
| 3 | enterGiveaway | 81.5% | api.ts |
...
```

---

### `ci remove <function-name> [path]`

Mark a dead function as **removed** (after you've deleted it).

```bash
# Remove by function name (partial match supported)
ci remove publishGiveaway

# Remove with commit hash
ci remove publishGiveaway --commit abc123def

# Remove in specific project
ci remove publishGiveaway ~/Documents/Kyma
```

**What it does:**
- Finds the matching pending function
- Marks it as "Removed" in outcomes
- Updates the outcome file

---

### `ci keep <function-name> "reason" [path]`

Mark a function as **false positive** (kept in codebase).

```bash
# Mark as false positive with reason
ci keep uploadImage "Used in tests"

# Keep in specific project
ci keep execute ~/Documents/Kyma "Discord bot command"
```

**Common reasons:**
- `"Used in tests"`
- `"Called via reflection"`
- `"External SDK integration"`
- `"Framework requirement"`
- `"Code is dead but planned for future"`

---

### `ci stats [path]`

View outcome statistics for a project.

```bash
# Stats for current project
ci stats

# Stats for specific project
ci stats ~/Documents/Kyma
```

**Output:**
```
📊 Outcome Statistics for: /home/dicey/Documents/X_giveaway_system

📊 Summary:
   Total flagged: 20
   Removed: 5 (25.0%)
   Kept (false positives): 3
   Pending: 12

💡 12 functions waiting for review. Run `ci list` to see them.
```

---

### `ci report [path]`

Generate a detailed report.

```bash
# Generate markdown report (default)
ci report

# Generate JSON report
ci report --format json

# Generate HTML report
ci report --format html

# Save to file
ci report --output report.md
```

---

### `ci train`

Train or retrain the ML model with new data.

```bash
# Train with default data
ci train

# Train with specific data
ci train --data data/train.json --output model_v3.bin
```

---

### `ci config`

Manage global configuration.

```bash
# Set config value
ci config set model /path/to/model.bin
ci config set threshold 0.55
ci config set verbose true

# Get config value
ci config get model
ci config get threshold

# List all config
ci config list
```

## Workflow Example

### Complete Workflow for a New Project

```bash
# 1. Navigate to project
cd ~/Documents/my-awesome-project

# 2. First analysis
ci analyze

# 3. Review results
ci list

# 4. Remove dead code (in your IDE/editor)
# Delete the functions flagged as dead

# 5. Mark them as removed
ci remove publishGiveaway
ci remove updateGiveaway
ci remove enterGiveaway

# 6. For functions you decide to keep
ci keep uploadImage "Used in tests"
ci keep verifyTasks "Called via reflection"

# 7. Check progress
ci stats

# 8. Re-analyze to find more
ci analyze

# 9. Generate final report
ci report --output dead_code_report.md
```

## Config File Location

Global config is stored at:
```
~/.config/code-intelligence/config.toml
```

Example config:
```toml
[defaults]
model = "/home/user/code-intelligence/model_verified_v2.bin"
threshold = 0.55
verbose = false

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

## Project Detection

`ci` auto-detects project type based on files present:

| File | Language |
|------|----------|
| `Cargo.toml` | Rust |
| `package.json` + `tsconfig.json` | TypeScript |
| `package.json` | JavaScript |
| `go.mod` | Go |
| `pom.xml` / `build.gradle` | Java |
| `requirements.txt` / `pyproject.toml` | Python |

## Supported Languages

- Rust
- TypeScript
- JavaScript (with JSX/TSX)
- Python
- Go
- Java
- More coming...

## Threshold Guide

| Threshold | Use Case | Precision | Recall |
|-----------|----------|-----------|--------|
| `0.55-0.60` | **Auto-removal** (safe to delete automatically) | ~100% | Low |
| `0.45-0.50` | **Manual review** (high confidence) | ~80% | Medium |
| `0.35-0.40` | **Code audit** (find all potential dead code) | ~77% | High |

### Recommended Settings

```bash
# For safety (no false positives)
ci config set threshold 0.55

# For finding more dead code (some false positives)
ci config set threshold 0.40

# For specific projects
ci analyze --threshold 0.40  # Project-specific override
```

## Troubleshooting

### "No model configured"

```bash
ci config set model ~/Documents/code-intelligence/model_verified_v2.bin
```

### "No tracked outcomes found"

Run analysis first:
```bash
ci analyze
```

### "No pending function found matching 'x'"

Check the exact function name:
```bash
ci list
```

### "Command not found: ci"

Reinstall:
```bash
cargo install --path . --bin ci
```

Or run from the project directory:
```bash
cargo run --bin ci -- analyze
```

## Advanced Tips

### 1. Automate with Git Hooks

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

### 2. Run on Multiple Projects

```bash
# Check all your projects
for project in ~/Documents/*; do
    if [ -d "$project" ]; then
        echo "📊 Analyzing: $project"
        ci analyze "$project" --threshold 0.55 2>/dev/null
    fi
done
```

### 3. Generate Team Report

```bash
ci report --format markdown --output ~/Desktop/dead_code_report.md
```

## Uninstall

```bash
cargo uninstall ci

# Also remove config
rm -rf ~/.config/code-intelligence
```

## License

MIT

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request

## Support

- Issues: [GitHub Issues](https://github.com/yourusername/code-intelligence/issues)
- Documentation: [Wiki](https://github.com/yourusername/code-intelligence/wiki)
