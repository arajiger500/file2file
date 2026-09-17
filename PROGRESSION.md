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

## 2026-09-16 22:30 (High-Fidelity Document Conversion)
- **Branch:** main
- **Objective:** Implement universal image preservation for document conversions, focusing on PDF → DOCX.
- **Implemented:**
    - Upgraded PDF → DOCX pipeline: Replaced `pdftotext` with `pdftohtml -c -dataurls -noframes` followed by Pandoc.
    - Added `pdftohtml` as a new Tauri sidecar binary.
    - Updated `sidecar.rs` for `pdftohtml` probing and health diagnostics.
    - Updated `run_pandoc_conversion` with flags for universal image preservation (`--embed-resources`, `--extract-media`).
    - Improved error mapping in `errors.rs` for Poppler utilities.
    - Added integration tests for image and table preservation in `src-tauri/tests/`.
- **Tests Performed:**
    - `cargo test --test image_preservation_test` (Passed).
    - `cargo test --test table_preservation_test` (Passed).
- **Improvements:**
    - PDF to DOCX now correctly embeds images, diagrams, and logos.
    - DOCX to HTML now embeds images as data URLs.
    - Markdown to DOCX now handles embedded base64 images correctly.
- **Next Actions:**
    - Audit security of shell execution (Pandoc/FFmpeg/ImageMagick sidecar configs).
    - Implement OCR support for scanned PDFs (Phase 2, Task 10).

## 2026-09-17 19:50 (Universal Content Preservation)
- **Branch:** main
- **Objective:** Audit and upgrade all conversion paths to preserve all meaningful embedded content.
- **Implemented:**
    - **FFmpeg**: Added universal stream mapping (`-map 0`), metadata preservation (`-map_metadata 0`), and chapter mapping (`-map_chapters 0`).
    - **Pandoc**: Added high-fidelity math support (`+tex_math_dollars`) and ensured media extraction/embedding for all document paths.
    - **ImageMagick**: Implemented transparency flattening for JPEG and frame coalescence for animated formats.
    - **Tests**: Added `torture_tests.rs` verifying transparency, metadata, multi-stream, and math preservation.
- **Improvements:**
    - Multi-track MKVs now convert to MP4 without losing audio/subtitles.
    - MP3 tags are preserved during audio conversions.
    - Transparent logos no longer turn black when converted to JPEG.
    - Complex academic Markdown/PDFs preserve math formulas.
- **Tests Performed:**
    - `cargo test --test torture_tests` (All 4 passed).
- **Next Actions:**
    - Audit security of shell execution (specifically FFmpeg complex filters).
    - Implement Phase 2, Task 10 (MOBI support).
