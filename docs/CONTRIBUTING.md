# Contributing to Code Intelligence

Thank you for your interest in contributing to `code-intelligence`! This document provides guidelines and instructions for contributing to the project.

---

## Code of Conduct

By participating in this project, you agree to:

- Be respectful and inclusive
- Provide constructive feedback
- Focus on the code, not the person
- Follow the project's coding standards

---

## Getting Started

### Prerequisites

- Rust 1.70+
- Cargo
- Git
- (Optional) Ollama, for LLM features

### Development Setup

```bash
# Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/code-intelligence
cd code-intelligence

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI
cargo run --bin ci -- --help
```

### Development Environment

```bash
# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Watch for changes
cargo watch -x check -x test

# Check test coverage
cargo tarpaulin --ignore-tests
```

---

## Project Structure

```
code-intelligence/
├── src/
│   ├── analysis/          # Analysis logic
│   ├── bin/                # CLI tools
│   ├── engine/             # Pipeline engine
│   ├── graph/               # Graph representations
│   ├── llm/                 # LLM integration
│   ├── ml/                   # Machine learning
│   ├── optimize/             # Optimization
│   ├── output/                # Output generation
│   ├── parser/                 # Parsing
│   └── utils/                  # Utilities
├── tests/                 # Tests
├── docs/                  # Documentation
├── models/                # ML models
├── data/                  # Training data
└── benches/                # Benchmarks
```

---

## Development Workflow

### 1. Find an Issue

Check the [issue tracker](https://github.com/neontoshi/code-intelligence/issues) for:

- Good first issues
- Bug reports
- Feature requests

### 2. Create a Branch

```bash
# Create a feature branch
git checkout -b feature/your-feature-name

# Or, for bug fixes
git checkout -b fix/issue-number
```

### 3. Make Changes

**Coding Standards:**

```rust
// Use descriptive variable names
let function_count = call_graph.node_count();

// Add documentation for public items
/// Analyzes dead code in a project
pub fn analyze_dead_code(project: &Path) -> Result<DeadCodeReport, Error> {
    // ...
}

// Handle errors properly
match result {
    Ok(data) => process_data(data),
    Err(e) => return Err(format!("Failed to process: {}", e)),
}

// Use the error taxonomy
return Err(CodeIntelError::ParseError {
    path: file_path.clone(),
    source: Box::new(e),
});
```

### 4. Write Tests

```rust
// tests/unit/your_module.rs
#[test]
fn test_function_works() {
    let result = my_function();
    assert_eq!(result, expected_value);
}

// tests/integration/your_feature.rs
#[test]
fn test_end_to_end_flow() {
    let result = run_analysis(test_project_path());
    assert!(result.is_ok());
}
```

### 5. Run Tests

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_function_works

# Run with output
cargo test -- --nocapture
```

### 6. Format and Lint

```bash
# Format code
cargo fmt

# Check lints
cargo clippy -- -D warnings

# Fix lints automatically
cargo clippy --fix
```

### 7. Commit Changes

```bash
# Commit with a descriptive message
git commit -m "feat: add support for new language"

# Commit format
# feat:     new feature
# fix:      bug fix
# docs:     documentation
# refactor: code refactoring
# test:     adding or updating tests
# chore:    maintenance
```

### 8. Create a Pull Request

1. Push your branch
2. Open a pull request
3. Fill in the PR template
4. Wait for review

---

## Coding Standards

### Rust Style Guide

**Naming:**

```rust
// Types: PascalCase
struct MyStruct;
enum MyEnum;
trait MyTrait;

// Functions: snake_case
fn my_function() {}

// Variables: snake_case
let my_variable = 42;

// Constants: SCREAMING_SNAKE_CASE
const MAX_FUNCTIONS: usize = 10000;

// Modules: snake_case
mod my_module;
```

**Imports:**

```rust
// Standard library first
use std::path::PathBuf;
use std::collections::HashMap;

// External crates next
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Internal modules last
use crate::analysis::DeadCodeDetector;
use crate::graph::CallGraph;
```

**Error Handling:**

```rust
// Use the defined error types
use crate::error::CodeIntelError;

// Prefer Result over panic
pub fn parse_file(path: &Path) -> Result<ParsedFile, CodeIntelError> {
    // ...
}

// Use context for better errors
path.canonicalize()
    .map_err(|e| CodeIntelError::IoError { source: e })?;
```

**Documentation:**

```rust
/// Analyzes dead code in a project.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory
/// * `config` - Analysis configuration
///
/// # Returns
///
/// Returns a `DeadCodeReport` containing all dead code findings.
///
/// # Errors
///
/// Returns `CodeIntelError` if:
/// - The project path doesn't exist
/// - Analysis fails
/// - The memory limit is exceeded
///
/// # Example
///
/// ```
/// let report = analyze_dead_code(
///     Path::new("./my-project"),
///     &Config::default(),
/// )?;
/// ```
pub fn analyze_dead_code(
    project_path: &Path,
    config: &Config,
) -> Result<DeadCodeReport, CodeIntelError> {
    // ...
}
```

---

## Testing Guidelines

### Unit Tests

```rust
// tests/unit/module_test.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_output() {
        let input = "test";
        let result = process_input(input);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_error_case() {
        let result = process_invalid_input();
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
// tests/integration/feature_test.rs
use code_intelligence::Pipeline;

#[tokio::test]
async fn test_full_analysis() {
    let project = setup_test_project();
    let mut pipeline = Pipeline::new();
    let analysis = pipeline.process_project(&project).await.unwrap();

    assert!(analysis.function_count() > 0);
}
```

### Golden Tests

```rust
// tests/integration/golden_test.rs
#[test]
fn test_golden_output() {
    let input = load_fixture("input.rs");
    let result = analyze(&input);
    let expected = load_fixture("expected.json");
    assert_eq!(result, expected);
}
```

### Fuzz Tests

```rust
// tests/fuzz_tests.rs
#[test]
fn test_parser_fuzz() {
    for _ in 0..1000 {
        let code = generate_random_code();
        let result = parser.parse(&code);
        assert!(result.is_ok() || result.is_err());
    }
}
```

---

## Feature Development

### Adding a New Language

1. **Add the tree-sitter grammar:**
   ```toml
   # Cargo.toml
   [dependencies]
   tree-sitter-newlang = "0.20"
   ```

2. **Add the language config:**
   ```rust
   // src/parser/tree_sitter.rs
   langs.insert(
       "nl".to_string(),
       LanguageConfig {
           name: "NewLang".to_string(),
           extensions: vec!["nl".to_string()],
           language_fn: tree_sitter_newlang::language,
           function_kinds: vec!["function_declaration".to_string()],
           // ...
       },
   );
   ```

3. **Add language-specific filters:**
   ```rust
   // src/analysis/dead_code/filters.rs
   fn is_newlang_framework(func: &FunctionNode) -> bool {
       // Detect framework patterns
   }
   ```

4. **Add tests:**
   ```rust
   // tests/fixtures/adversarial/newlang/
   ```

### Adding a New Feature

1. **Define the feature:**
   ```rust
   // src/ml/feature_schema.rs
   features.push(FeatureDefinition {
       name: "new_feature".to_string(),
       index: index,
       description: "Description of new feature".to_string(),
       category: FeatureCategory::NewCategory,
       normalization: Normalization::MinMax { min: 0.0, max: 1.0 },
   });
   ```

2. **Extract the feature:**
   ```rust
   // src/analysis/features.rs
   impl FunctionFeatures {
       fn extract_new_feature(func: &FunctionNode) -> f64 {
           // Compute feature value
       }
   }
   ```

3. **Update model training:**
   ```rust
   // src/ml/classifier.rs
   // The schema handles this automatically
   ```

### Adding a New LLM Provider

1. **Create the provider:**
   ```rust
   // src/llm/providers/new_provider.rs
   use async_trait::async_trait;

   pub struct NewProvider {
       // ...
   }

   #[async_trait]
   impl LLMProvider for NewProvider {
       async fn generate(&self, messages: &[LLMMessage], options: &GenerationOptions) -> Result<LLMResponse, String> {
           // Implementation
       }

       async fn generate_stream(&self, messages: &[LLMMessage], options: &GenerationOptions) -> Result<Box<dyn Stream<Item = Result<String, String>> + Send>, String> {
           // Implementation
       }
   }
   ```

2. **Add it to the factory:**
   ```rust
   // src/llm/providers/mod.rs
   pub enum ProviderType {
       // ...
       NewProvider,
   }

   pub async fn create_provider(provider_type: ProviderType, config: &ProviderConfig) -> Result<Arc<dyn LLMProvider>, String> {
       match provider_type {
           // ...
           ProviderType::NewProvider => Ok(Arc::new(NewProvider::new(config)?)),
       }
   }
   ```

---

## Performance Guidelines

### Memory Usage

- **Avoid large clones**: use references instead
- **Use `Arc` for shared data**: reduces duplication
- **Use iterators**: avoid collecting large vectors
- **Limit graph size**: skip cycle detection for large graphs

### Time Complexity

- **O(N) parsing**: keep parsing linear
- **O(F + C) reachability**: BFS is efficient
- **O(F) ML prediction**: keep prediction fast

### Optimization Tips

```rust
// Bad: clone large data
let copy = data.clone();

// Good: use a reference
let data_ref = &data;

// Bad: collect a large vector
let vec: Vec<_> = iter.collect();

// Good: use the iterator directly
for item in iter { ... }

// Bad: expensive hash on every call
let hash = compute_expensive_hash(&data);

// Good: cache the hash
let hash = cache.get_or_compute(&key, || compute_expensive_hash(&data));
```

---

## Documentation Standards

### Code Documentation

```rust
//! Module-level documentation
//!
//! This module provides functionality for X.
//! It handles Y and Z.

/// Function documentation
///
/// # Arguments
/// * `arg` - Description
///
/// # Returns
/// * `Result<...>` - Description
pub fn my_function(arg: &str) -> Result<String, Error> {
    // ...
}
```

### Documentation Files

```
docs/
├── algorithm.md          # Algorithm explanation
├── limitations.md        # Known limitations
├── evaluation_report.md  # Model evaluation
├── api.md                # API reference
├── user_guide.md         # User guide
├── architecture.md       # Architecture overview
└── deployment.md         # Deployment guide

CONTRIBUTING.md            # This file (repo root)
```

> **Note:** the original listing nested `CONTRIBUTING.md` inside `docs/`, but every link in this file (e.g. `[docs/](docs/)`) treats `docs/` and the contributing guide as siblings — the standard convention is `CONTRIBUTING.md` at the repo root, next to `Cargo.toml`. Split it out above; flag if it's actually meant to live inside `docs/`.

---

## Review Process

### Pull Request Requirements

1. **Tests**: all new code must have tests
2. **Documentation**: public API must be documented
3. **No warnings**: code must compile without warnings
4. **No panics**: avoid `unwrap()` and `expect()` in production code

### Review Checklist

- [ ] Code follows the style guide
- [ ] Tests pass
- [ ] Documentation updated
- [ ] No breaking changes (or announced)
- [ ] Performance impact considered
- [ ] Error handling is complete

---

## Release Process

### Version Bumping

```toml
# Cargo.toml
version = "0.2.0"  # Major.Minor.Patch
```

### Release Checklist

- [ ] All tests pass
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped
- [ ] Release notes prepared
- [ ] Models updated

### Creating a Release

```bash
# Create a release branch
git checkout -b release/v0.2.0

# Update the version
sed -i 's/version = "0.1.0"/version = "0.2.0"/' Cargo.toml

# Commit changes
git commit -am "Release v0.2.0"

# Create a tag
git tag v0.2.0

# Push
git push origin release/v0.2.0
git push origin v0.2.0
```

---

## Getting Help

### Resources

- **Documentation**: [docs/](docs/)
- **Issue Tracker**: [GitHub Issues](https://github.com/neontoshi/code-intelligence/issues)
- **Discussions**: [GitHub Discussions](https://github.com/neontoshi/code-intelligence/discussions)

### Asking Questions

When asking for help, please provide:

1. What you're trying to do
2. What you've tried
3. Error messages (if any)
4. Your environment (OS, Rust version, etc.)

---

## Thank You!

Your contributions make this project better. Whether you're fixing a bug, adding a feature, or improving documentation, every contribution is valued.

**Happy coding! 🚀**
