# Code Intelligence Engine — Roadmap to 9.5/10

Work through phases **in order**. Don't start a phase until the previous one is working and tested.

```
PHASE 1 ML training correctness
PHASE 2 Ground truth + verification
PHASE 3 Evaluation + scientific validity
PHASE 4 Verdict engine + deletion safety
PHASE 5 Static/dynamic analysis quality
PHASE 6 Architecture + performance + reliability
PHASE 7 Product polish + documentation
PHASE 8 Final 9.5/10 validation gate
```

---

## 🔴 Phase 1 — ML training correctness(DONE)
*Goal: make sure the model is actually learning from the data you claim it's learning from.*

- [ ] Enforce `TrainingDataFilter` inside the real training path (currently `LinearClassifier::train()` only filters `Unknown`, so `StaticHeuristic` examples leak in)
- [ ] Exclude `StaticHeuristic` labels from training
- [ ] Exclude `Weak` labels from production training
- [ ] Decide whether `Silver` (0.30 weight, `is_verified() = false`) is genuinely trainable — consider gating it behind an "experimental training" flag rather than silently including it
- [ ] Apply `training_weight()` to the loss/gradient so weights actually affect learning (error × training_weight × feature)
- [ ] Fix training accuracy metric to use the trainable population only, not all non-`Unknown` examples
- [ ] Report label-source counts separately: total / trainable / verified / silver / excluded-heuristic / excluded-unknown
- [ ] Report verified accuracy, weighted training accuracy, and unweighted training accuracy separately
- [ ] Add a hard safety test: 1000 `StaticHeuristic` + 100 `HumanVerified` examples → only `HumanVerified` should influence the model
- [ ] Add a hard safety test: training on `StaticHeuristic`-only data should fail/refuse
- [ ] Fix threshold pipeline order: `TRAIN → CALIBRATE (validation) → TUNE threshold (calibrated validation predictions) → FREEZE → TEST once`
- [ ] Persist the frozen threshold into the saved model in a well-defined probability space
- [ ] Add automated checks preventing test data from entering calibration/tuning

## 🔴 Phase 2 — Ground truth(DONE)
*Goal: stop having the static analyzer teach the ML model what the static analyzer already believes.*

- [ ] Formalize the label hierarchy:
  - Level 4 — `ProductionVerified` (actual removal/outcome observed)
  - Level 3 — `HumanVerified` (human reviewed candidate)
  - Level 2 — `GitVerified` (symbol-level historical verification)
  - Level 1 — `DatasetVerified` (external/high-quality dataset)
  - Level 0 — `Silver` (strong but indirect evidence)
  - Excluded — `StaticHeuristic`, `Weak`
- [ ] Replace textual `GitVerified` check (grep + `git log -S`) with symbol-level verification: parse AST → find exact function → confirm disappearance at commit N+1 → check references before/after removal → confirm build/tests pass
- [ ] Introduce a real symbol identity (repository, module, file, language, qualified symbol, signature, hash) instead of matching on bare names
- [ ] Track symbol evolution (rename → move → signature change → removal) as one lineage where possible
- [ ] Strengthen `OutcomeTracker` as the real feedback loop: candidate → prediction → human decision → actual removal → post-removal outcome → training example
- [ ] Separate **label provenance** ("why do we believe this label") from **verdict provenance** ("why did the analyzer reach this conclusion")
- [ ] Build a verified dataset generation pipeline from the above

## 🔴 Phase 3 — Evaluation & scientific validity(DONE)
*Goal: prove the model works on unseen code, not that it reproduces your own heuristics.*

- [ ] Regenerate every benchmark for the 235-feature schema (docs/tests still reference 46 features)
- [ ] Build three baselines: static-only, ML-only, static+ML+dynamic/framework fusion — report Precision / Recall / F1 / FPR for each and confirm fusion actually beats static
- [ ] Make repository-isolated testing mandatory (never train and test on the same repo)
- [ ] Add temporal testing (train on past commits, test on later commits)
- [ ] Add a fully unseen-repositories benchmark (train on repos A–J, test on K–N)
- [ ] Add per-language benchmarks (Rust, Python, TypeScript, Java, Go, C/C++, ...)
- [ ] Add class-imbalance metrics: Precision, Recall, F1, PR-AUC, FPR, FNR, Specificity — prioritize precision (false positive = telling a dev safe code is dead)
- [ ] Rebuild the ablation study across feature groups (all / graph-only / lexical-only / complexity-only / type-only / framework-only / dynamic-only / leave-one-out variants / static-only / ML-only / fusion)
- [ ] Add leakage tests: repo identity, file path, generated code, symbol name, duplicate functions, same-project commits across train/test
- [ ] Add calibration evaluation: ECE / Brier / log-loss before and after calibration; compare temperature / histogram / isotonic methods, choosing on validation and evaluating on test

## 🟠 Phase 4 — Verdict engine & deletion safety(DONE)
*Goal: turn "the model thinks this is dead" into "the system has enough independent evidence to recommend removal."*

- [ ] Make `Unknown` a real, central safety state (e.g. dynamic dispatch + no static callers + high ML dead probability → `Unknown`, not `DefinitelyDead`)
- [ ] Add evidence conflict resolution (conflicting static/ML/framework/dynamic signals → `UNKNOWN/REVIEW`, not majority vote)
- [ ] Define hard deletion blockers: FFI, reflection, dynamic loading, plugin/framework registration, macro-generated references, public API/external symbols, runtime config references, unknown call targets, conflicting evidence
- [ ] Separate verdict from deletion recommendation: `Verdict → Safety checks → Deletion recommendation`
- [ ] Create explicit, separate confidence scores: "dead confidence" vs "deletion safety"
- [ ] Build adversarial tests for dangerous combinations: dead+FFI, dead+reflection, dead+macro, dead+plugin, dead+framework, dead+public API, dead+dynamic import, alive+no static callers, alive+test-only caller, alive+generated caller

## 🟠 Phase 5 — Analyzer quality(DONE)
*Goal: improve the evidence feeding the verdict engine.*

- [ ] Audit every root-detection category: main, tests, benchmarks, public API, exported symbols, framework entrypoints, CLI commands, handlers, routes, plugins, FFI, generated code, build scripts, config references
- [ ] Split dynamic-reference detection into separate systems (reflection, dynamic imports, FFI, function pointers, trait/interface dispatch, DI, macros, generated code, runtime registration, IPC), each with its own confidence level
- [ ] Track evidence as structured `DynamicReference { kind, confidence, source, location }` instead of a boolean
- [ ] Build a framework registry (roots, decorators, annotations, registration patterns, generated entrypoints, dynamic behavior) instead of scattered framework checks
- [ ] Normalize semantic concepts across all language adapters (Function, Method, Class, Module, Call, Reference, Root, Export, DynamicReference)
- [ ] Add cross-language parity fixtures (obviously dead / alive / public API / dynamic reference / framework entrypoint / recursive function / function pointer / generated code) and compare analyzer behavior across languages

## 🟡 Phase 6 — Architecture, performance & reliability
*Goal: handle genuinely large repositories.*

- [ ] Standardize all errors on `CodeIntelError` with variants: Parse, Graph, Model, Dataset, Cache, Config, Git, IO, Framework, Evaluation
- [ ] Remove `Result<T, String>` and `Box<dyn Error>` usage in favor of `CodeIntelError`
- [ ] Audit every `unwrap()` / `expect()` / `panic!()` / `unreachable!()`; categorize as safe-invariant / test-only / user-controlled / production-runtime; remove unsafe production cases
- [ ] Store model/schema compatibility metadata (model version, feature schema version, feature count, feature names/hash, training dataset version, timestamp, git commit, calibration method, threshold) and reject incompatible models on load
- [ ] Make analysis incremental (changed files → changed ASTs → affected graph nodes → recompute only the affected region)
- [ ] Add content-hash-based caching for AST, file hashes, function features, call graph, dynamic references, framework analysis, ML features, Git symbol history
- [ ] Benchmark on large repos (10k / 50k / 100k / 500k / 1M+ functions), measuring parse time, graph construction, feature extraction, ML inference, memory, disk cache, total runtime
- [ ] Add concurrency limits (worker count, queue size, memory usage) instead of spawning a task per file

## 🟢 Phase 7 — Product polish & documentation
*Goal: turn a technically strong engine into a polished tool.*

- [ ] Fix every stale "46 features" reference (README, ML docs, evaluation docs, architecture docs, CLI docs, examples, benchmarks, reports)
- [ ] Make evaluation reproducible via a single command (e.g. `ci evaluate --train ... --val ... --test ... --output evaluation/`) producing dataset manifest, model, calibration, threshold, test results, confusion matrix, per-language results, feature importance, ablation results
- [ ] Version datasets (dataset-v1, v2, v3...) with source repos, commit SHAs, label counts/sources, languages, generation date, filters
- [ ] Version experiments: model, schema, dataset, training commit, validation/test split, threshold, calibration method
- [ ] Improve CLI per-candidate output: verdict, dead confidence, deletion safety, evidence checklist, recommendation
- [ ] Add machine-readable output formats: JSON, JSONL, SARIF
- [ ] Add a stable, documented CI mode (`ci check .` with `--max-dead`, `--max-risk`, `--fail-on-definitely-dead`, `--fail-on-high-confidence`)
- [ ] Clean the repo: unused modules, duplicate classifiers, old experiments, stale binaries, dead feature flags, duplicate eval tools, old docs, temp scripts

## 🏆 Phase 8 — Final 9.5/10 validation gate
*No new features here — just prove the system.*

- [ ] **Gate 1 — Ground truth:** model trained primarily on independently verified labels, not labels from the same heuristic system being evaluated
- [ ] **Gate 2 — Generalization:** demonstrated on unseen repositories, future commits, and multiple languages
- [ ] **Gate 3 — ML contribution:** demonstrate Fusion > Static on the metrics that matter
- [ ] **Gate 4 — Safety:** no critical dynamic/framework/FFI case is ever auto-recommended for deletion
- [ ] **Gate 5 — Calibration:** stated confidence (e.g. 90%) matches empirical correctness within acceptable error
- [ ] **Gate 6 — Reproducibility:** someone else can clone the project and reproduce the benchmark
- [ ] **Gate 7 — Scale:** analysis remains practical on large repositories

---

## Immediate priority

**Start with Phase 1.** Most of the architecture (`TrainingDataFilter`, `training_weight()`) already exists — it just isn't enforced by the actual learning loop yet. Fixing that gives a clean foundation for everything downstream. Don't touch Phases 5–8 until 1–3 are solid (expect ~9.2–9.4 range after Phase 3).
