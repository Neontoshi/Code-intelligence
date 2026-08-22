#!/bin/bash
set -euo pipefail

cd ~/Documents/code-intelligence

echo "🚀 Complete Training Pipeline"
echo "============================="
echo ""

# Step 1: Show data stats
echo "📊 Data Statistics:"
echo "   Train: $(cat data/train.json | jq 'length') examples"
echo "   Validation: $(cat data/val.json | jq 'length') examples"
echo "   Test: $(cat data/test.json | jq 'length') examples"

# Step 2: Train
echo ""
echo "🧠 Training model..."
mkdir -p models
cargo run --release --bin train_model -- \
    --train-data data/train.json \
    --val-data data/val.json \
    --output models/dead_code_model_v3.bin \
    --target-precision 0.95

# Step 3: Calibrate
echo ""
echo "🔬 Calibrating model..."
cargo run --release --bin calibrate_model -- \
    --model models/dead_code_model_v3.bin \
    --val-data data/val.json \
    --output models/dead_code_model_v3_calibrated.bin \
    --method temperature

# Step 4: Evaluate
echo ""
echo "📊 Evaluating model..."
cargo run --release --bin evaluate_metrics -- \
    --model models/dead_code_model_v3_calibrated.bin \
    --test-data data/test.json \
    --output evaluation_results.json

# Step 5: Show results
echo ""
echo "📊 Evaluation Results:"
cat evaluation_results.json | jq '.'

# Step 6: Set default
echo ""
echo "⚙️ Setting as default model..."
ci config set model models/dead_code_model_v3_calibrated.bin

# Step 7: Quick test
echo ""
echo "🧪 Quick test on codebase..."
ci analyze . --threshold 0.80 --max-files 50

echo ""
echo "✅ Training complete!"
echo "📁 Model: models/dead_code_model_v3_calibrated.bin"
echo "📊 Results: evaluation_results.json"
