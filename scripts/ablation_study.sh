#!/bin/bash
set -euo pipefail

echo "🔬 Feature Ablation Study"
echo "========================="
echo ""

mkdir -p ablation_results

# Full model baseline
echo "📊 Training full model..."
cargo run --release --bin train_model \
    -- --train-data data/train.json \
    --val-data data/val.json \
    --output ablation_results/full_model.bin

# Feature subsets
for subset in "graph" "signature" "name" "file" "type" "complexity"; do
    echo ""
    echo "📊 Training with ${subset} features only..."
    cargo run --release --bin feature_ablation \
        -- --train-data data/train.json \
        --val-data data/val.json \
        --feature-set "$subset" \
        --output "ablation_results/${subset}_model.bin"
done

echo ""
echo "📊 Generating ablation report..."
cargo run --release --bin ablation_report \
    -- --results-dir ablation_results \
    --output ablation_results/ablation_report.md

echo ""
echo "✅ Ablation study complete!"
echo "📁 Results: ablation_results/ablation_report.md"
