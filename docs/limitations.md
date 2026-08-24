
```markdown
# Limitations

## Overview

While `code-intelligence` provides high-precision dead code detection by blending AST analysis, call graphs, and calibrated machine learning, static analysis of dynamic codebases has inherent boundaries[cite: 1, 2]. This document details the tool's limitations, edge cases across supported languages, and recommended mitigations[cite: 1, 2].

---

## 1. Reflection & Dynamic Dispatch

### Problem

Runtime string lookups and reflection bypass static call graph construction[cite: 1, 2]:

```python
# Python reflection - string target cannot be inferred statically in arbitrary expressions
func = getattr(module, dynamic_var)
func()

```

```go
// Go reflection
v := reflect.ValueOf(obj)
method := v.MethodByName(dynamicMethodName)
method.Call(nil)

```

```java
// Java reflection
Method method = obj.getClass().getMethod(methodName);
method.invoke(obj);

```

```php
// PHP variable function calls
$funcName = getAction();
$funcName();

```

### Impact

Functions invoked exclusively via reflection or runtime string lookups will register zero static incoming callers (`fan_in = 0`) and risk being marked as dead candidates.

### Detection

The engine includes AST pattern extractors in `src/analysis/dynamic_refs.rs` for common reflection signatures:

* **Python**: `getattr(obj, "string_literal")`

* **Go**: `reflect.MethodByName("Literal")`

* **PHP**: `call_user_func("string_literal")`


Literal patterns are extracted and added to the reachability graph. Dynamic variables (e.g., `getattr(obj, computed_str)`) can only be flagged as dynamic call sites, not resolved to concrete symbols.

### Mitigation

1. **Keep / Whitelist**: Mark the candidate in the dashboard (`f`) or CLI:
```bash
ci keep dynamicMethod "Invoked via reflection in serializer"

```


2. **Annotation**: Add comments or docstrings containing `reflection` or `dynamic_dispatch` to lower ML confidence scores.



---

## 2. Polymorphic Interfaces & Trait Objects

### Problem

Polymorphism and virtual method tables decouple call sites from exact implementations:

```rust
// Rust dynamic dispatch
let handler: Box<dyn Handler> = get_handler();
handler.handle(); // Static graph cannot always pinpoint the exact struct implementor

```

```csharp
// C# interface resolution via DI container
public interface IOrderService { void Process(); }
// Injected at runtime via MediatR / ServiceProvider

```

### Impact

Concrete implementations of interface methods may have zero direct callers in the call graph.

### Mitigation

The engine includes safety rules in `is_never_dead()` and root detection:

* **Rust**: Methods inside `impl Trait for Type` blocks are never classified as `DefinitelyDead`.


* **C# / Java**: Classes decorated with `@Service`, `@Repository`, `[ApiController]`, or implementing registered interfaces are preserved as potential entry points.



---

## 3. FFI (Foreign Function Interface) & Native Exports

### Problem

Functions exported across language boundaries (e.g., Rust to C/WASM, C++ JNI exports) are called by external runtimes:

```rust
#[no_mangle]
pub extern "C" fn native_compute(ptr: *const u8, len: usize) -> i32 {
    // ...
}

```

```cpp
extern "C" JNIEXPORT void JNICALL Java_com_app_Native_run(JNIEnv* env, jobject obj) {
    // ...
}

```

### Impact

FFI endpoints have no internal callers within the analyzed codebase.

### Detection & Mitigation

The root detector inspects symbols for FFI patterns (`#[no_mangle]`, `extern "C"`, `JNIEXPORT`, `EMSCRIPTEN_KEEPALIVE`, `Q_INVOKABLE`) and automatically registers them as external roots.

---

## 4. Metaprogramming & Macros

### Problem

Procedural and declarative macros generate AST nodes after macro expansion:

```rust
#[derive(Serialize, Deserialize)]
struct State {
    id: String,
}
// Macros expand into helper serialization traits/methods invisible in unexpanded AST

```

### Mitigation

* Generated files matching common patterns (`*.gen.rs`, `*_gen.go`, `*.pb.go`, `*_pb2.py`) are flagged with `is_generated = true`.


* Symbols derived from macros receive reduced dead-code scores.



---

## 5. Dynamic Module Loading & IPC

### Problem

Runtime dynamic imports and cross-process messaging hide invocation targets:

```typescript
// Dynamic import
const module = await import(`./plugins/${pluginName}`);
module.initialize();

// Electron / Tauri IPC bridge
window.__TAURI__.invoke('sync_database', { payload });
ipcRenderer.send('download-complete', data);

```

### Detection & Mitigation

The AST parser identifies string-literal IPC calls (`invoke`, `send`, `emit`) and maps them to backend Rust/C++/Node handler functions automatically. Non-literal dynamic paths must be reviewed manually.

---

## 6. Monolith Scalability (>100k Functions)

### Performance Envelope

| Metric | Target / Limit | Behavior Exceeded |
| --- | --- | --- |
| **Call Graph Nodes** | ~5,000 | Cycle detection skips beyond 5,000 nodes to preserve O(V + E) complexity

 |
| **RAM Usage** | 4,096 MB | Configurable via `CI_MEMORY_LIMIT_MB`<br> |
| **File Traversal** | 10,000 files | Exceeding files require `--max-files` override or incremental cache

 |

### Mitigation

* **Incremental Analysis**: Pass `--cache` to cache file AST hashes (`.code-intelligence-cache`).


* **Scope Reduction**: Analyze submodules or specific packages rather than whole multi-gigabyte monolith repositories.



---

## Summary Table

| Limitation | Impact | Severity | Mitigation Strategy |
| --- | --- | --- | --- |
| **Reflection**<br> | False dead candidates

 | High

 | AST literal extraction + manual whitelist

 |
| **Polymorphism / DI**<br> | Indirect method usage

 | Medium

 | Interface and trait methods protected

 |
| **FFI / WASM Exports**<br> | Zero internal callers

 | High

 | Automated attribute export detection

 |
| **Macros / CodeGen**<br> | Unparsed expansions

 | Medium

 | Generated file markers + reduced penalty

 |
| **IPC & Dynamic Imports**<br> | Decoupled execution

 | Medium

 | IPC literal AST matching

 |
| **Large Graphs (>5k nodes)**<br> | Slower graph traversals

 | Low

 | Cycle skipping + disk caching

 |

---

## Best Practices

1. **Review Before Deleting**: Use the interactive dashboard (`ci dashboard .`) to inspect evidence, caller paths, and confidence intervals before removing code.


2. **Record Decisions**: When keeping a false positive, record the reason with `ci keep <name> "<reason>"`. This writes to `.code-intelligence-outcomes.json` and supplies training feedback for future model iterations.


3. **Calibrate for Codebase Type**:
* Libraries / SDKs: Use `--threshold 0.92` (protect public API entry points).


* Applications / Services: Use `--threshold 0.80 - 0.85` (aggressive dead logic pruning).




4. **Run Test Suites**: Always execute your automated test pipeline after pruning flagged symbols.



> **Rule of thumb**: Keeping dead code carries technical debt, but deleting dynamically dispatched code causes runtime failures. If confidence is below `0.85` or evidence is uncertain, review the candidate before removal.
> 
> 

```

```
