#!/bin/bash

# File2File Repository Cleanup Script
# This script removes all generated artifacts and temporary files.

echo "Cleaning up File2File repository..."

# Directories to remove
JUNK_DIRS=(
    "matrix_test_output"
    "stress_batch_output"
    "stress_batch_files"
    "test-output"
    "test-results"
    "benchmark-results"
    "coverage"
    "debug-output"
    "qa-output"
    "conversion-output"
    ".artifacts"
)

# Files to remove
JUNK_FILES=(
    "matrix_report.md"
    "matrix_report.json"
    "src-tauri/*.png"
    "*.tmp"
    "*.temp"
    "*.log"
    "*.bak"
    "*.orig"
    "*.cache"
)

for dir in "${JUNK_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo "Removing directory: $dir"
        rm -rf "$dir"
    fi
done

for pattern in "${JUNK_FILES[@]}"; do
    # Use find to handle patterns safely
    find . -maxdepth 2 -name "$pattern" -exec echo "Removing file: {}" \; -exec rm -f {} +
done

# Deep clean src-tauri
if [ -d "src-tauri/target" ]; then
    echo "Cleaning Rust target directory..."
    # cd src-tauri && cargo clean && cd ..
    # Or just rm -rf src-tauri/target
    rm -rf src-tauri/target
fi

echo "Repository is clean."
