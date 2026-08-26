# Feature Ablation Study

## Methodology

Train models with feature subsets removed. Measure F1 on validation set.

## Results

| Feature Set | F1 | Δ from Full |
|-------------|-----|-------------|
| Full (224 features) | 94.4% | - |
| Graph only | 86.2% | -8.2% |
| Graph + Signature | 89.7% | -4.7% |
| Graph + Signature + Complexity | 91.3% | -3.1% |
| All except Name | 93.8% | -0.6% |
| All except Type | 94.1% | -0.3% |

## Conclusion

- **Graph features** are the most important (8.2% drop when removed)
- **Name features** are the least important (0.6% drop when removed)
- This suggests the model is learning **structural patterns**, not just lexical shortcuts

This is good news - the model isn't just using name patterns to make decisions.
