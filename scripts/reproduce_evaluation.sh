#!/bin/bash
set -euo pipefail

echo "🔬 Reproducing Evaluation Results"
echo "================================="
echo ""

# 1. Verify dataset
echo "📊 Verifying dataset..."
if [ ! -f "data/manifest.json" ]; then
    echo "❌ Dataset manifest not found!"
    exit 1
fi

if [ ! -f "data/train.json" ] || [ ! -f "data/val.json" ] || [ ! -f "data/test.json" ]; then
    echo "❌ Dataset files missing!"
    exit 1
fi

# 2. Verify model
echo "📊 Verifying model..."
if [ ! -f "models/manifest.json" ]; then
    echo "❌ Model manifest not found!"
    exit 1
fi

if [ ! -f "models/dead_code_model_v2.bin" ]; then
    echo "❌ Model file missing!"
    exit 1
fi

# 3. Run evaluation
echo "📊 Running evaluation..."
cargo run --release --bin evaluate_metrics \
    -- --model models/dead_code_model_v2.bin \
    --test-data data/test.json \
    --output evaluation_results.json

# 4. Compare with reported metrics
echo "📊 Comparing results..."
python3 -c "
import json

with open('evaluation_results.json') as f:
    actual = json.load(f)

with open('models/manifest.json') as f:
    reported = json.load(f)['performance']

print('\nComparison:')
print('  Metric      | Reported | Actual')
print('  ------------+----------+-------')
for key in ['accuracy', 'precision', 'recall', 'f1']:
    diff = abs(actual[key] - reported[key])
    status = '✅' if diff < 0.01 else '⚠️'
    print(f'  {key:11} | {reported[key]:.3f}    | {actual[key]:.3f}   {status}')
"

echo ""
echo "✅ Reproducibility check complete!"
