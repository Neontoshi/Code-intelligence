# Code Intelligence Models

## Dead Code Detection Models

### dead_code_model_v1.bin
- **Version**: v1
- **Training Data**: 2,159 examples
- **Accuracy**: 74.8%
- **Features**: 33
- **Notes**: Original model with test functions labeled as DEAD

### dead_code_model_v2.bin
- **Version**: v2
- **Training Data**: 2,159 examples
- **Accuracy**: 74.8%
- **Features**: 33
- **Notes**: Fixed test function labels (test functions = ALIVE)

## Duplicate Detection Models

### duplicate_model_v1.bin
- **Version**: v1
- **Training Data**: 18,924 examples (actix-web)
- **Accuracy**: 22.4%
- **Notes**: Three labels (Duplicate/Similar/Not) - needs improvement

### duplicate_model_v2.bin (coming soon)
- **Version**: v2
- **Training Data**: Balanced dataset
- **Accuracy**: Expected 80-90%
- **Notes**: Binary labels (Duplicate/NotDuplicate)

## Usage

```bash
# Dead code detection
cargo run --bin dead_code_check -- ~/project --model models/dead_code_model_v2.bin

# Duplicate detection
cargo run --bin dedup_check -- ~/project --duplicate-model models/duplicate_model_v2.bin
