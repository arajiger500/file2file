# Progression Journal

## 2026-09-04 22:35 (Folder Drop Support)
- **Branch:** main
- **Objective:** Enable dragging and dropping entire folders for ZIP archiving.
- **Implemented:**
    - Updated `DropZone.tsx` to detect directories and assign them the `archive` category with a `directory` extension.
    - Updated `formats.rs` to recognize the `directory` extension as part of the `Archive` category.
    - Verified that folders now correctly trigger ZIP archive suggestions in the UI.
- **Tests Performed:**
    - Full build (`npm run build` & `cargo check`).
- **Next Actions:**
    - Implement better PDF to DOCX using `pdftohtml` as an intermediate step.
    - Add more detailed comparison notes for formats in `formats.rs`.
    - Implement "Clear History" functionality in the UI if not already fully functional.

## 2026-09-04 23:35 (Bug Fixes: JSON-CSV & Zip)
- **Branch:** fix/conversion-bugs
- **Objective:** Fix broken JSON-CSV conversion and optimize Zip performance.
- **Implemented:**
    - Rewrote JSON to CSV logic to manually handle headers and mapping (fixed `rust-csv` limitation).
    - Optimized Zip archiving to use `io::copy` (streamed) instead of loading entire files into memory.
    - Added robust error handling and removed `.unwrap()` calls in Zip extraction logic.
    - Added regression tests for JSON-CSV (mixed keys, empty array) and Zip roundtrip.
- **Tests Performed:**
    - `cargo test` (All 4 unit tests passed).
- **Bugs Fixed:**
    - Fixed "serializing maps is not supported" error in JSON-CSV conversion.
    - Fixed potential OOM crash in Zip archiving.
    - Fixed potential backend crash in Zip extraction.
- **Security Improvements:**
    - Added alphanumeric sanitization for `target_format` to prevent path traversal.
    - Replaced unsafe `.unwrap()` on file paths with proper UTF-8 validation and error propagation.
    - Updated `converter.rs` to be generic over `tauri::Runtime` for better testability with `MockRuntime`.
- **Browser-Side Fixes:**
    - Fixed naive JSON-CSV conversion in `universal-engine.ts` (now handles mixed keys and escaping).
    - Improved CSV-JSON parsing in `universal-engine.ts` to handle basic quoted fields.
- **Next Actions:**
    - Audit security of shell execution (Pandoc/FFmpeg/ImageMagick sidecar configs).
    - Implement PDF to DOCX improvements.
