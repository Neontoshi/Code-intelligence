#!/bin/bash
set -euo pipefail

echo "📊 Collecting Training Data from Multiple Projects"
echo "=================================================="
echo ""

# Create directories
mkdir -p training_repos
mkdir -p training_data

# List of Rust projects to analyze
PROJECTS=(
    "https://github.com/rust-lang/rust-clippy.git"
    "https://github.com/serde-rs/serde.git"
    "https://github.com/tokio-rs/tokio.git"
    "https://github.com/actix/actix-web.git"
    "https://github.com/diesel-rs/diesel.git"
    "https://github.com/rayon-rs/rayon.git"
    "https://github.com/rust-lang/rust-analyzer.git"
    "https://github.com/async-rs/async-std.git"
)

# Also add Python/JS/Go projects for multi-language support
# (uncomment if you want to include them)
# PROJECTS+=(
#     "https://github.com/pallets/flask.git"
#     "https://github.com/django/django.git"
#     "https://github.com/facebook/react.git"
#     "https://github.com/golang/go.git"
# )

echo "📊 Will process ${#PROJECTS[@]} repositories"
echo ""

# Process each project
for repo_url in "${PROJECTS[@]}"; do
    repo_name=$(basename "$repo_url" .git)
    repo_dir="training_repos/$repo_name"

    echo "📦 Processing: $repo_name"

    # Clone if not exists
    if [ ! -d "$repo_dir" ]; then
        echo "   Cloning $repo_url..."
        git clone --depth 1 "$repo_url" "$repo_dir" 2>/dev/null || {
            echo "   ⚠️ Failed to clone $repo_name, skipping..."
            continue
        }
    fi

    # Generate training data
    output_file="training_data/${repo_name}.json"
    echo "   Generating training data..."

    if cargo run --release --bin training_data_exporter "$repo_dir" "$output_file" 2>/dev/null; then
        count=$(cat "$output_file" | jq 'length' 2>/dev/null || echo "0")
        echo "   ✅ Generated $count examples"
    else
        echo "   ⚠️ Failed to generate training data for $repo_name"
        rm -f "$output_file"
    fi

    echo ""
done

# Merge all training data
echo "📊 Merging all training data..."
cargo run --release --bin merge_all_training_data

echo ""
echo "✅ Training data collection complete!"
echo "📁 Results: training_data/combined_training.json"
