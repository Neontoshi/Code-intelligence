#!/bin/bash
set -euo pipefail

cd ~/Documents/code-intelligence

echo "🚀 Training Duplicate Detection Model"
echo "===================================="
echo ""

# Step 1: Train on combined data
echo "📊 Training on combined training data..."
cargo run --release --bin train_duplicate_model -- \
    training_data/combined_training.json \
    models/duplicate_model_v3.bin

# Step 2: Check model
echo ""
echo "📁 Model saved:"
ls -lh models/duplicate_model_v3.bin

# Step 3: Test on code-intelligence
echo ""
echo "🧪 Testing deduplication on code-intelligence..."
cargo run --release --bin dedup_check -- . --threshold 0.85 --ml

echo ""
echo "✅ Duplicate training complete!"
echo "📁 Model: models/duplicate_model_v3.bin"
