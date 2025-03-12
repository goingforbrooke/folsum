#!/bin/zsh

# Run benchmarks with the bench feature enabled
cargo bench --features bench

# Get the current date in YY-MM-DD format
DATE=$(date +"%y-%m-%d")

# Define the target directory (adjust relative path if needed)
TARGET_DIR="benchmarks/${DATE}_historical_benchmark"

# Create the target directory if it doesn't exist
mkdir -p "$TARGET_DIR"

# Copy all benchmark artifacts from ../target/criterion/ into the target directory
cp -r ../target/criterion/* "$TARGET_DIR"

echo "Benchmark results have been archived to $TARGET_DIR"

cargo flamegraph --root --bench benchmark_directory_summarization --features bench -- --bench

mv flamegraph.svg "$TARGET_DIR"
