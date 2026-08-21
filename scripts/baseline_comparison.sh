#!/bin/bash
set -euo pipefail

echo "📊 Baseline Comparison"
echo "======================"
echo ""

mkdir -p baseline_results

# Heuristic baseline (static analysis only)
echo "📊 Running heuristic baseline..."
cargo run --release --bin dead_code_check . \
    --no-ml \
    --threshold 0.80 \
    --output-report baseline_results/heuristic_report.json

# Logistic Regression (our model)
echo "📊 Running logistic regression..."
cargo run --release --bin evaluate_metrics \
    -- --model models/dead_code_model_v2.bin \
    --test-data data/test.json \
    --output baseline_results/logistic_results.json

# Random Forest (if we add it)
echo "📊 Running random forest..."
cargo run --release --bin train_random_forest \
    -- --train-data data/train.json \
    --val-data data/val.json \
    --output baseline_results/random_forest.bin

# Gradient Boosting (if we add it)
echo "📊 Running gradient boosting..."
cargo run --release --bin train_gradient_boosting \
    -- --train-data data/train.json \
    --val-data data/val.json \
    --output baseline_results/gradient_boosting.bin

echo ""
echo "✅ Baseline comparison complete!"
echo "📁 Results: baseline_results/"
