#!/bin/bash
set -euo pipefail

echo "📊 Running Performance Benchmarks"
echo "=================================="
echo ""

mkdir -p benchmark_results

# Define test repositories
repos=(
    "small:https://github.com/rust-lang/rustfmt.git"
    "medium:https://github.com/rust-lang/rust-clippy.git"
    "large:https://github.com/rust-lang/rust.git"
    "huge:https://github.com/rust-lang/rust-analyzer.git"
)

for repo_spec in "${repos[@]}"; do
    size="${repo_spec%%:*}"
    url="${repo_spec#*:}"
    name=$(basename "$url" .git)

    echo ""
    echo "📊 Benchmarking $size repository: $name"

    # Clone if not exists
    if [ ! -d "benchmark_repos/$name" ]; then
        echo "  Cloning $url..."
        git clone --depth 1 "$url" "benchmark_repos/$name"
    fi

    # Cold run
    echo "  Cold analysis..."
    /usr/bin/time -f "  Time: %e seconds, Memory: %M KB" \
        cargo run --release --bin dead_code_check \
        "benchmark_repos/$name" \
        --threshold 0.80 \
        --no-cache \
        --max-files 10000 \
        2>&1 | tee "benchmark_results/${size}_cold.log"

    # Warm run (with cache)
    echo "  Warm analysis..."
    /usr/bin/time -f "  Time: %e seconds, Memory: %M KB" \
        cargo run --release --bin dead_code_check \
        "benchmark_repos/$name" \
        --threshold 0.80 \
        --cache \
        --max-files 10000 \
        2>&1 | tee "benchmark_results/${size}_warm.log"

    # Incremental (if supported)
    echo "  Incremental analysis..."
    touch "benchmark_repos/$name/src/lib.rs"
    /usr/bin/time -f "  Time: %e seconds, Memory: %M KB" \
        cargo run --release --bin dead_code_check \
        "benchmark_repos/$name" \
        --threshold 0.80 \
        --cache \
        --max-files 10000 \
        2>&1 | tee "benchmark_results/${size}_incremental.log"
done

echo ""
echo "✅ Benchmarks complete!"
echo "📁 Results: benchmark_results/"
