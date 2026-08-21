## Document 2: `docs/limitations.md`

```markdown
# Limitations

## Overview

While `code-intelligence` is a powerful tool for dead code detection, it has inherent limitations. This document honestly describes what the tool **can't do** and how to work around these limitations.

---

## 1. Reflection

### Problem

Languages like Python, Go, and Java support reflection - calling functions by name at runtime:

```python
# Python reflection - static analysis cannot see this call
func = getattr(module, "dynamic_function_name")
func()
```

```go
// Go reflection
v := reflect.ValueOf(obj)
method := v.MethodByName("DynamicMethod")
method.Call(nil)
```

```java
// Java reflection
Method method = obj.getClass().getMethod("dynamicMethod");
method.invoke(obj);
```

### Impact

Functions called via reflection **appear dead** because there are no static call edges pointing to them.

### Detection

The tool attempts to detect reflection patterns:

```rust
// Detects reflection usage
if body.contains("getattr(") || body.contains("reflect.") {
    // Flag as dynamic reference
}
```

But this only marks the **presence** of reflection, not the actual targets.

### Mitigation

1. **Whitelist**: Manually add reflection targets to the whitelist
2. **Documentation**: Document reflection usage in code comments
3. **Annotation**: Use `#[cfg(not(dead_code_check))]` for reflection targets

---

## 2. Dynamic Dispatch

### Problem

Languages with dynamic dispatch (trait objects, interfaces, virtual methods) can hide call relationships:

```rust
// Rust trait object
let handler: Box<dyn Handler> = Box::new(DefaultHandler);
handler.handle();  // Static analysis can't see which implementation
```

```cpp
// C++ virtual methods
Animal* animal = new Dog();
animal->speak();  // Calls Dog::speak(), but static analysis doesn't know
```

### Impact

Trait implementations may appear dead even though they're used polymorphically.

### Mitigation

The tool **never marks trait implementations as dead**:

```rust
fn is_never_dead(func: &FunctionNode) -> bool {
    if func.trait_impl.is_some() {
        return true;  // Trait implementations are safe
    }
    // ...
}
```

---

## 3. FFI (Foreign Function Interface)

### Problem

Functions exported to other languages (C, Python, etc.) have callers outside the codebase:

```rust
// Rust FFI - called from C
#[no_mangle]
pub extern "C" fn process_data(data: *const u8) -> i32 {
    // ...
}
```

```c
// C code calling the Rust function
int result = process_data(buffer);
```

### Impact

FFI functions appear dead because there are no internal callers.

### Detection

The tool detects FFI exports:

```rust
fn is_ffi_export(func: &FunctionNode) -> bool {
    func.name.contains("extern") ||
    func.doc_comment.contains("#[no_mangle]") ||
    func.file.contains("/ffi/")
}
```

### Mitigation

FFI functions are **never marked dead** by the filter pipeline.

---

## 4. Macros

### Problem

Macros generate code that isn't visible to static analysis:

```rust
// Macro-generated code
#[derive(Debug, Clone, Serialize)]
struct Config {
    name: String,
    value: i32,
}

// The macro generates:
// - impl Debug for Config
// - impl Clone for Config
// - impl Serialize for Config
```

### Impact

Macro-generated functions appear dead because the macro expansion isn't parsed.

### Detection

The tool looks for macro indicators:

```rust
if func.doc_comment.contains("macro") ||
   func.file.contains(".gen.rs") ||
   func.file.contains("_gen.rs") {
    // Likely macro-generated
}
```

### Mitigation

Generated code patterns are added to the never-dead filter.

---

## 5. Dynamic Imports

### Problem

JavaScript/TypeScript dynamic imports are resolved at runtime:

```javascript
// Dynamic import - invisible to static analysis
const module = await import('./dynamic-module.js');
module.dynamicFunction();
```

```python
# Python dynamic import
import importlib
module = importlib.import_module('dynamic_module')
module.dynamic_function()
```

### Impact

Functions in dynamically imported modules appear dead.

### Detection

The tool detects dynamic import patterns:

```rust
if body.contains("import(") || body.contains("importlib") {
    // Flag as dynamic import
}
```

### Mitigation

Dynamic import usage is flagged but not fully resolved.

---

## 6. Large Codebases

### Problem

Very large codebases (>100k functions, >1M LOC) can cause performance issues:

- **Memory**: Building the full call graph can exceed 4GB
- **Time**: Analysis can take 10+ minutes
- **Cycle Detection**: O(N²) algorithms become impractical

### Mitigations

#### Memory Limits

```rust
// Configurable memory limit
config.max_memory_mb = Some(4096);  // 4GB limit

// Automatic degradation
if current_memory > limit * 0.85 {
    // Reduce threads, skip expensive features
}
```

#### Incremental Analysis

```rust
// Only re-analyze changed files
let changed = file_tracker.detect_changes(&files);
if changed.is_empty() {
    return cached_analysis;  // Fast path
}
```

#### Cycle Detection Skipping

```rust
// Skip cycle detection for huge graphs
if call_graph.node_count() > 5000 {
    skip_cycle_detection = true;
}
```

---

## 7. Generated Code

### Problem

Generated code (protobuf, bindings, etc.) often has patterns that look dead:

```rust
// protobuf-generated code
pub struct User {
    #[prost(string, tag="1")]
    pub name: String,
}

// Generated methods that look dead
impl User {
    pub fn name(&self) -> &str { &self.name }
    pub fn set_name(&mut self, name: String) { self.name = name; }
}
```

### Impact

Generated accessors and builders appear dead but are used by the code generator.

### Detection

The tool skips files with generated markers:

```rust
fn is_generated_file(file: &str) -> bool {
    file.contains(".gen.rs") ||
    file.contains("_gen.rs") ||
    file.contains(".pb.go") ||
    file.contains("_pb2.py") ||
    file.contains("/generated/") ||
    file.contains("/gen/")
}
```

### Mitigation

Generated files are partially skipped or have features disabled.

---

## 8. Testing Code

### Problem

Test code often has functions that appear dead but are called by the test runner:

```rust
#[test]
fn test_helper() {
    // Called by cargo test
}

#[bench]
fn bench_parser() {
    // Called by cargo bench
}
```

### Impact

Test functions may appear dead if the test runner isn't detected.

### Mitigation

Test functions are automatically detected and marked as roots:

```rust
fn is_test_function(func: &FunctionNode) -> bool {
    func.is_test ||
    func.name.starts_with("test_") ||
    func.file.contains("/tests/") ||
    func.file.ends_with("_test.rs")
}
```

---

## 9. Build System Integration

### Problem

Build scripts, benchmarks, and examples have their own entry points:

```
build.rs    - Called by cargo build
benches/    - Called by cargo bench
examples/   - Called by cargo run --example
```

### Impact

Functions in these directories may appear dead.

### Mitigation

These paths are automatically added to roots:

```rust
if file.contains("/benches/") ||
   file.contains("/examples/") ||
   file.ends_with("build.rs") {
    // Treat as root
}
```

---

## Summary Table

| Limitation | Severity | Mitigation |
|------------|----------|------------|
| Reflection | High | Dynamic reference detection |
| Dynamic Dispatch | Medium | Never mark trait impls dead |
| FFI | High | Detect FFI exports |
| Macros | Medium | Detect generated patterns |
| Dynamic Imports | Medium | Detect import patterns |
| Large Codebases | Low | Memory limits, incremental |
| Generated Code | Low | Skip generated files |
| Test Code | Low | Auto-detect test functions |
| Build System | Low | Auto-detect build paths |

---

## Best Practices

### For Developers

1. **Use Attributes**: `#[cfg(not(dead_code_check))]` for reflection targets
2. **Add Doc Comments**: Document FFI and reflection usage
3. **Organize Code**: Put generated code in `generated/` directories
4. **Review Verdicts**: Always review before removing code
5. **Test After Removal**: Run tests after deleting dead code

### For Team Leads

1. **Set Thresholds**: Use `--threshold` for PR checks
2. **Track Outcomes**: Use `ci stats` to monitor progress
3. **Retrain Models**: Periodically retrain with new data
4. **Document Exceptions**: Keep a project whitelist

---

## Known False Positives

### Common False Positives

| Pattern | Example | Why It's Flagged |
|---------|---------|------------------|
| Trait methods | `impl Handler for DefaultHandler` | No direct calls |
| Framework hooks | `@app.route('/')` | Called by framework |
| FFI exports | `#[no_mangle]` | Called from external |
| Test helpers | `fn setup_test()` | Called by tests |
| Build scripts | `build.rs` | Called by cargo |

### How to Handle

```bash
# Mark as false positive
ci keep setup_test "Used by integration tests"

# Or add to whitelist permanently
ci config set whitelist "setup_test,prepare_db,create_test_data"
```

---

## Disclaimer

No static analysis tool can be 100% accurate. Always **review before removing** code, and **run tests** after making changes.

> **Rule of thumb**: If you're not sure, keep it. Removing a used function is worse than keeping a dead one.
```
