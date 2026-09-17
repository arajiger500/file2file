# File2File Production Validation & Release Testing Plan

This plan outlines the implementation of a comprehensive, automated testing suite to validate every conversion path, handle "torture testing" of edge cases, and ensure production-grade reliability across all platforms.

## User Review Required

> [!IMPORTANT]
> The automated matrix test will require the presence of system binaries (`ffmpeg`, `magick`, `pandoc`, `pdftotext`) on the testing environment. If these are missing, those specific conversion paths will be marked as "Skipped/System Missing" in the report.

> [!WARNING]
> **Huge File Testing (10GB+):** These tests will be disabled by default in CI and must be triggered manually or run on dedicated hardware to avoid disk space and timeout issues.

## Proposed Changes

### 1. Automated Inventory System (The Matrix)

I will implement a Rust-based tool (as a new test module) that programmatically generates the complete conversion matrix by iterating through all supported extensions and their compatible targets.

#### [MODIFY] [formats.rs](file:///home/ara/projects/file2file/file2file/src-tauri/src/formats.rs)
- Expose a `get_all_extensions()` function to allow the test runner to discover the full scope of the application.

#### [NEW] [matrix_tests.rs](file:///home/ara/projects/file2file/file2file/src-tauri/src/matrix_tests.rs)
- A new test suite that:
    1. Generates the full matrix of `Input A -> Output B`.
    2. Maps each pair to a sample file in `test_files/`.
    3. Executes the conversion.
    4. Validates the output using probes (e.g., `ffprobe` for media, `stat` for size, parsing for JSON/CSV).
    5. Generates a `matrix_report.json` and a human-readable `matrix_report.md`.

### 2. File Name Torture Testing

#### [NEW] [torture_tests.rs](file:///home/ara/projects/file2file/file2file/src-tauri/src/torture_tests.rs)
- A specialized test suite that creates files with:
    - Extremely long paths (MAX_PATH limits).
    - Unicode/Emoji names (🔥, 中文, Türkçe).
    - Special characters (spaces, multiple dots, shell-sensitive chars).
- Validates that the backend handles these paths correctly without crashing or failing to locate files.

### 3. Media & Large File Stress Testing

#### [NEW] [stress_tests.rs](file:///home/ara/projects/file2file/file2file/src-tauri/src/stress_tests.rs)
- Implements tests for:
    - **Large Batch:** Converting 100+ files simultaneously.
    - **Resolution Torture:** Validating 8K video and huge TIFF images.
    - **Memory Tracking:** Measuring memory usage during long conversions (30 min+).

### 4. Hardware Acceleration & Fallback

#### [MODIFY] [converter.rs](file:///home/ara/projects/file2file/file2file/src-tauri/src/converter.rs)
- Enhance the hardware acceleration tests to mock GPU failure and verify that the system automatically falls back to CPU encoding without failing the job.

### 5. Automated CI/CD Integration

#### [NEW] [test_summary.py](file:///home/ara/projects/file2file/file2file/scripts/test_summary.py)
- A script to aggregate Rust test results and generate a "Release Readiness" dashboard.

---

## Verification Plan

### Automated Tests
- Run `cargo test --test matrix_tests` to validate the full conversion matrix.
- Run `cargo test --test torture_tests` to validate file system edge cases.
- Run `cargo test --test stress_tests` (with `--ignored` flag for huge files).

### Manual Verification
- Verify that the `matrix_report.md` correctly lists all supported pairs and their status.
- Trigger a build on Windows, macOS, and Linux to verify cross-platform binary compatibility.
