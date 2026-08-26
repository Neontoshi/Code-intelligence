Here's the complete updated `models/README.md`:

```markdown
# Code Intelligence Models

This directory contains trained ML models for dead code detection.

## Current Model

### model.bin (v3.0)
- **Status**: ✅ Production ready
- **Features**: 235 features
- **Schema Version**: 1
- **Calibration**: Temperature scaling
- **Training Data**: Verified labels only (Phase 1+)
- **Performance**: See `manifest.json` for latest metrics
- **Threshold**: 0.80 (see `manifest.json` for authoritative value)

## Model History

| Version | Date | Status | Notes |
|---------|------|--------|-------|
| v3.0 | 2026-08-26 | ✅ Current | 235 features, verified labels |
| v2.1 | 2026-08-21 | ❌ Obsolete | 46 features, heuristic labels |
| v1 | 2026-08-22 | ❌ Obsolete | Initial version |

## Training a New Model

```bash
# 1. Collect verified training data
cargo run --bin verify_ground_truth -- --interactive

# 2. Train with validation data
cargo run --bin train -- model \
    --data data/train.json \
    --val-data data/val.json \
    --output models/model.bin \
    --precision 0.95
```

The `--val-data` flag is now **required** for proper training:
- Validates model performance during training
- Calibrates probability outputs
- Tunes threshold to achieve target precision

## Evaluation Report

To reproduce the evaluation report:

```bash
cargo run --bin phase2_evaluation \
    --train-data data/train.json \
    --val-data data/val.json \
    --test-data data/test.json \
    --output-dir evaluation_results
```

The report will be generated at:
- `evaluation_results/phase2_results.json` - Full metrics
- `evaluation_results/phase2_report.md` - Human-readable report

The evaluation includes:
1. Feature ablation study (235 features)
2. Repository-isolated evaluation
3. Temporal evaluation (if timestamps available)
4. Static vs ML vs Static+ML comparison
5. Calibration metrics (ECE, Brier score, log loss)

## Cleaning Up Obsolete Models

```bash
# See what would be removed
cargo run --bin cleanup_models -- --dry-run

# Actually remove obsolete models
cargo run --bin cleanup_models
```

## Using the Model

### Dead Code Detection

```bash
# Direct usage
cargo run --bin dead_code_check -- ~/project --model models/model.bin

# Via CI command
ci analyze ~/project --model models/model.bin
```

### Safety Notes

⚠️ **The model NEVER recommends deletion by itself.**

The verdict system enforces:
- **Dynamic/runtime evidence overrides ML predictions** - if code is used at runtime, it's marked alive regardless of ML
- **Evidence conflict detection** - disagreements between ML, static, and dynamic evidence are flagged
- **No ML-only deletions** - deletion recommendations require BOTH static AND ML agreement
- **Human verification required** - all deletion recommendations need human review

### Understanding Verdicts

| Verdict | Meaning | Action |
|---------|---------|--------|
| 🟢 DEFINITELY ALIVE | Strong evidence of usage | Keep |
| 🟡 PROBABLY ALIVE | Some evidence of usage | Keep |
| ⚪ UNKNOWN | Insufficient evidence | Review required |
| 🟠 PROBABLY DEAD | Some evidence of dead code | Review required |
| 🔴 DEFINITELY DEAD | Strong evidence of dead code | Review required before deletion |

**Note**: Even "DEFINITELY DEAD" requires human review before deletion. The system is conservative by default.

## Feature Schema

Current schema version: **1** (235 features)

The feature schema is versioned to ensure compatibility:
- Models trained with schema v1 will only work with tools using schema v1
- Schema changes require retraining
- Check `src/ml/feature_schema.rs` for the authoritative feature list

## Troubleshooting

### "Model schema mismatch"
The model was trained with a different feature schema. Retrain with the current schema.

### "Model not found"
Ensure the model path is correct and the file exists.

### "Poor performance on my code"
- Run `cargo run --bin phase2_evaluation` to check performance
- Check per-language performance with `analyze_features_per_language`
- Consider collecting more verified training data from your repositories

### "Model calibration is poor"
- Run `cargo run --bin train -- calibrate --model model.bin --data data/val.json`
- Check the ECE metric in the evaluation report

## Data Requirements

Training data must use **verified labels only**:
- ✅ HumanVerified (developer reviewed)
- ✅ GitVerified (function removal confirmed)
- ✅ ProductionVerified (telemetry confirmed)
- ✅ DatasetVerified (curated benchmark)
- ⚠️ Silver (multiple heuristics agree - weak signal)
- ❌ StaticHeuristic (NOT for training)
- ❌ Weak (NOT for training)

This prevents circular training where the model learns from its own predictions.
```
```
