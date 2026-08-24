# Code Intelligence Models

This directory contains trained ML models for dead code and duplicate detection.

## Model Format

Models are stored as versioned JSON files containing:
- Model weights and bias
- Feature schema (ensures compatibility)
- Calibration parameters (if calibrated)
- Training metadata (dataset info, date, etc.)
- Performance metrics (accuracy, precision, recall, F1)
- Threshold settings

## Dead Code Detection Models

### model.bin
- **Version**: v1
- **Status**: ✅ Production ready
- **Features**: 46 features (graph, signature, name, file, type, complexity)
- **Calibration**: Temperature scaling
- **Performance** (on held-out test set):
  - Accuracy: 95.3%
  - Precision: 96.8%
  - Recall: 92.1%
  - F1: 94.4%
  - FPR: 2.1%
- **Training Data**: 15,847 examples from 23 repositories
- **Languages**: Rust, Python, TypeScript, Go, Java
- **Threshold**: 0.80 (optimal F1 balance)
- **Created**: 2026-08-22

### Legacy Models (v1-v3)
These are kept for reference but are no longer recommended for production use.
Use `model.bin` for best results.

## Duplicate Detection Models

### duplicate_model_v4.bin
- **Version**: v4
- **Status**: ✅ Production ready
- **Features**: 101 features (function signatures + type context)
- **Training Data**: Balanced dataset from multiple repositories
- **Threshold**: 0.70
- **Languages**: Rust, Python, TypeScript, Go, Java

## How to Use

### Dead Code Detection
```bash
# Use the recommended v4 model
cargo run --bin dead_code_check -- ~/project --model models/model.bin

# Or via the CI command
ci analyze ~/project --model models/model.bin
```

### Duplicate Detection
```bash
cargo run --bin dedup_check -- ~/project --duplicate-model models/duplicate_model_v4.bin

# Or via the CI command
ci dedup ~/project --ml --duplicate-model models/duplicate_model_v4.bin
```

## Model Training

To train a new model:
```bash
# Collect training data
cargo run --bin collect_training_data

# Merge and split data
cargo run --bin merge_all_training_data
cargo run --bin split_repositories

# Train model
cargo run --bin train_model -- --train-data data/train.json --val-data data/val.json

# Calibrate model
cargo run --bin calibrate_model -- --model model.bin --val-data data/val.json

# Tune threshold
cargo run --bin tune_threshold -- --model model.bin --val-data data/val.json

# Evaluate model
cargo run --bin evaluate_metrics -- --model model.bin --test-data data/test.json
```

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v1 | 2026-08-22 | Calibrated, balanced dataset, 46 features |

## Performance Comparison

| Model | Accuracy | Precision | Recall | F1 | FPR |
|-------|----------|-----------|--------|-----|-----|
| v4 | 95.3% | 96.8% | 92.1% | 94.4% | 2.1% |
| v3 | 93.7% | 94.8% | 89.6% | 92.1% | 3.2% |
| v2 | 74.8% | - | - | - | - |
| v1 | 74.8% | - | - | - | - |

## Troubleshooting

### "Model schema mismatch"
The model was trained with a different feature schema. Retrain with current schema.

### "Model not found"
Ensure the model path is correct and the file exists.

### "Poor performance on my code"
- Consider retraining on your specific codebase
- Run `ci evaluate-lang` to check per-language performance
- Collect more training data from your repositories
```

### model.bin
...
- **Threshold**: 0.92 (optimal F1 balance)
...
## Threshold Policy

The model manifest contains the authoritative threshold. When this model is loaded:

1. The model's threshold is used by default
2. Users can override with `--threshold` flag
3. The threshold is documented in the model manifest

---
