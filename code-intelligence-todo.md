# code-intelligence — Things to Change

Based on a full pass over the codebase (146 `src/` files, ~40k LOC). Ordered by priority.

---

## P0 — Architecture drift

### 1. `ci.rs` shells out to sibling binaries instead of calling the library

**Where:** `src/bin/ci.rs` (2,368 lines — largest file in the repo)

Your `docs/architecture.md` shows the CLI layer calling the Pipeline Engine directly. What's
actually there: most `run_*` handlers (`run_analyze`, `run_dedup`, `run_train`, `run_calibrate`,
`run_tune`, `run_export`, `run_merge`, `run_collect`, `run_dashboard`, ~15 more) call
`find_binary("dead_code_check")` etc. and spawn it as a subprocess via `std::process::Command`,
then just check the exit status.

```rust
// src/bin/ci.rs, run_analyze()
let binary_path = find_binary("dead_code_check");
let status = if let Some(bin_path) = binary_path {
    let mut cmd = std::process::Command::new(&bin_path);
    cmd.arg(path).args(["--model", &model_path]) ...
    cmd.status()?
} else {
    eprintln!("❌ Could not find 'dead_code_check' binary. ...");
    return Ok(());
};
```

`find_binary` (bottom of `ci.rs`) searches: cwd → `~/.cargo/bin` → `$PATH` → exe's own dir. If
none of those has the binary, the command silently no-ops with `Ok(())`.

**Why it matters:**
- Fragile — `ci` only works if sibling binaries happen to be built and discoverable.
- Argument parsing/orchestration is duplicated between `ci.rs` and each binary it calls.
- Errors collapse to an exit-status bool — no structured error ever reaches the caller.
- Can't unit-test `run_analyze` without actually spawning a real subprocess binary.

**Fix:** Make `run_analyze`/`run_dedup`/etc. call the library functions in `engine::pipeline` /
`analysis::dead_code` directly, the way `docs/architecture.md` describes. Drop `find_binary` and
the subprocess calls entirely once each handler is wired to the library.

---

### 2. `CodeIntelError` (`src/error.rs`) is defined but essentially unused

**Where:** `src/error.rs` defines a ~40-variant `CodeIntelError` enum via `thiserror`, plus
`ErrorContext` / `ErrorWithContext` / a `context_err!` macro.

**Actual usage found:**
- `CodeIntelError` / `error::Result` — used in **1** file (`error.rs` itself).
- `context_err!` macro — called **0** times.
- Meanwhile the rest of the codebase returns:
  - `Result<(), Box<dyn std::error::Error>>` — 64 occurrences
  - `Result<T, String>` / `Result<(), String>` — 23+ occurrences
  - `anyhow` — imported in exactly 1 file (`main.rs`)

**Why it matters:** You paid the design cost of a real error taxonomy (parse errors, graph
errors, model errors, cache errors, etc.) without getting any of the benefit — nothing can
match on error kind, and `?` conversions are inconsistent across modules.

**Fix — pick one:**
- **(a)** Migrate `engine/`, `analysis/`, `graph/`, `ml/` to return `error::Result<T>` /
  `CodeIntelError`, and have the `bin/*.rs` entry points convert to `anyhow`/`Box<dyn Error>`
  at the top level only.
- **(b)** Or delete `error.rs`'s unused machinery and standardize everything on `anyhow`.

Don't leave both in place.

---

## P1 — Structural bloat

### 3. 25 near-duplicate CLI binaries (~8,300 LOC in `src/bin/`, >20% of the codebase)

**Where:** `src/bin/*.rs` — e.g. `evaluate_metrics.rs` (920 lines), `evaluate_per_language.rs`
(104 lines), `evaluate_per_language_detailed.rs` (443 lines) are three separate binaries doing
overlapping work; same pattern with `train_model.rs` / `train_duplicate_model.rs` /
`calibrate_model.rs` / `tune_threshold.rs`.

Each repeats the same shape: its own `clap::Parser` struct, its own `main()`, its own
println-style progress output, then a call into the library.

**Fix:** Collapse related binaries into one with subcommands/flags (e.g. one `evaluate` binary
with a `--per-language` / `--detailed` flag instead of three binaries). Fewer entry points to
keep in sync when the underlying library API changes.

---

### 4. Dead-code detector has 26 unsuppressed `#[allow(dead_code)]` in its own source

**Where (concentration points):**
- `src/llm/providers/openai.rs` — 8 occurrences (unused struct fields, lines 42–100)
- `src/llm/providers/ollama.rs` — 4 occurrences (lines 48–72)
- `src/bin/common/exit_codes.rs` — 6 occurrences
- `src/parser/tree_sitter.rs`, `src/graph/resolver.rs`, `src/engine/cache.rs`,
  `src/optimize/dedup/candidates.rs`, `src/bin/temporal_evaluation.rs`, `src/bin/common/cleanup.rs`
  — 1 each

**Fix:** Run the tool on itself. For each `#[allow(dead_code)]`: either wire the field/fn up to
something that uses it, or delete it. If a few are genuinely intentional (e.g. reserved for a
planned provider feature), leave a comment saying why instead of a bare allow.

---

## P2 — Coverage gaps

### 5. Inline unit tests exist in only 6 of 146 `src/` files

**Files with `#[cfg(test)]`:** `engine/cache.rs`, `llm/providers/mock.rs`, `ml/feature_schema.rs`,
`ml/classifier.rs`, `bin/common/exit_codes.rs`, `optimize/dedup/minhash.rs`.

**Files with none, despite being core logic:** `graph/resolver.rs`, `graph/call_graph.rs`,
`analysis/dead_code/analyzer.rs`, `analysis/roots.rs`, `parser/tree_sitter.rs`,
`parser/semantic.rs`, `engine/pipeline.rs`, `engine/call_graph_builder.rs`, `analysis/dynamic_refs.rs`.

You do have a separate `tests/` dir with integration, property, fuzz, and adversarial-fixture
tests — that's good and covers end-to-end behavior. But the modules doing the heaviest lifting
have no unit-level tests, so a regression inside (say) `resolver.rs`'s resolution logic has to
be caught by an integration test noticing the wrong final output, rather than a focused test
pinpointing the broken function.

**Fix:** Add `#[cfg(test)] mod tests` to the modules above, focused on the tricky logic paths
(cycle handling in `resolver.rs`, dynamic-ref pattern matching, root detection edge cases).

---

## P3 — Smaller cleanups

### 6. Repeated struct-literal construction in `analysis/dynamic_refs.rs`

**Where:** Lines ~90–140. The same shape is built inline 3+ times:

```rust
DynamicReference {
    source_file: file.path.clone(),
    source_function: Some(func_info.name.clone()),
    target_function: Some(func_info.name.clone()),
    target_full_path: resolved_path.clone(),
    target_pattern: decorator.clone(),
    ...
}
```

Not a performance problem — the clones are legitimate (you need owned data). It's a DRY smell:
a small constructor/builder function would cut the duplication and prevent one call site
drifting out of sync with the others as fields get added later.

---

## Not a problem (checked, ruled out)

- `unwrap()`/`expect()` usage — 19 total, almost all in tests or one-time static regex
  compilation (`Regex::new(...).unwrap()` on constant patterns). Clean.
- The one `unsafe` block (`src/utils/alloc.rs`) has a proper `// SAFETY:` justification.
- `clone()` is used a lot (561 calls) but the hotspots checked (`dynamic_refs.rs`,
  `indexer.rs`, `roots.rs`) are cloning small owned data into result structs, not obvious
  perf bugs — no action needed beyond #6 above.

---

## Suggested order of attack

1. Fix `ci.rs` → direct library calls (P0 #1) — unblocks testing `ci`'s handlers at all.
2. Pick and enforce one error strategy (P0 #2).
3. Consolidate the binaries (P1 #3) — easier once #1 is done since the shared logic will
   already be in the library, not duplicated per-binary.
4. Clear the `#[allow(dead_code)]` list (P1 #4).
5. Backfill unit tests on the modules in #5 as you touch them.
