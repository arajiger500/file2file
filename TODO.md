# Priority TODO List

## Core Infrastructure
- [x] Verify build health (`npm run build`, `cargo check`).
- [x] Audit current hardware acceleration detection (Updated to use sidecar).
- [x] Audit security of file path handling and shell execution (Verified safe .args() usage).
- [x] Ensure all binaries (magick, pdftotext) are handled as sidecars.

## Conversion Implementation
### Phase 2: Document Conversions (1 to 15)
- [x] 1. PDF to DOCX (Via pdftotext + Pandoc)
- [x] 2. DOCX to PDF (Via Pandoc)
- [x] 3. PDF to TXT (Via pdftotext)
- [x] 4. PDF to HTML (Via pdftotext + Pandoc)
- [x] 5. HTML to PDF (Via Pandoc)
- [x] 6. Markdown to PDF (Via Pandoc)
- [x] 7. Markdown to HTML (Via Pandoc)
- [x] 8. EPUB to PDF (Via Pandoc)
- [x] 9. EPUB to TXT (Via Pandoc)
- [ ] 10. MOBI to EPUB (Requires ebook-convert/Calibre)
- [x] 11. RTF to DOCX (Via Pandoc)
- [x] 12. DOCX to TXT (Via Pandoc)
- [x] 13. ODT to PDF (Via Pandoc)
- [x] 14. ODT to DOCX (Via Pandoc)
- [x] 15. CSV to JSON (Implemented via Rust native)

### Phase 3: Data, Spreadsheet & Structured Conversions (16 to 30)
- [x] 16. CSV to XLSX (Implemented via rust_xlsxwriter)
- [x] 17. XLSX to CSV (Implemented via calamine)
- [x] 18. XLSX to JSON (Implemented via calamine)
- [x] 19. JSON to CSV
- [x] 20. JSON to YAML
- [x] 21. YAML to JSON
- [x] 22. XML to JSON (Implemented via quick-xml)
- [x] 23. JSON to XML (Implemented via manual serializer)
- [x] 24. TOML to JSON
- [x] 25. JSON to TOML
- [x] 26. SQLite DB to JSON dump (Implemented via rusqlite)
- [x] 27. CSV to SQL Insert statements (Implemented via Rust native)
- [x] 28. Log file to structured JSON (Implemented via Regex parser)
- [x] 29. BibTeX to JSON (Implemented via Regex parser)
- [x] 30. ICS Calendar to JSON (Implemented via Rust native)

### Phase 4: Image & Raster Graphics Conversions (31 to 45)
- [x] 31. PNG to JPEG (Via ImageMagick)
- [x] 32. JPEG to PNG (Via ImageMagick)
- [x] 33. PNG to WebP (Via ImageMagick)
- [x] 34. WebP to PNG (Via ImageMagick)
- [x] 35. PNG to AVIF (Via ImageMagick)
- [x] 36. AVIF to PNG (Via ImageMagick)
- [x] 37. SVG to PNG (rasterization) (Via ImageMagick)
- [x] 38. HEIC to JPEG (Via ImageMagick)
- [x] 39. TIFF to PNG (Via ImageMagick)
- [x] 40. BMP to JPEG (Via ImageMagick)
- [x] 41. GIF to WebP animation (Via ImageMagick)
- [x] 42. PNG to ICO (multi-resolution icon generation) (Implemented via FFmpeg filters)
- [x] 43. RAW camera format to JPEG preview (Via FFmpeg)
- [x] 44. PSD thumbnail extraction to PNG (Via ImageMagick)
- [x] 45. SVG to optimized minimal SVG (Requires svgo sidecar - Partially via magick)

### Phase 5: Audio, Video & Multimedia Conversions (46 to 60+)
- [x] 46. MP4 to MKV (Via FFmpeg)
- [x] 47. MKV to MP4 (Via FFmpeg remuxing)
- [x] 48. AVI to MP4 (Via FFmpeg)
- [x] 49. MOV to MP4 (Via FFmpeg)
- [x] 50. WebM to MP4 (Via FFmpeg)
- [x] 51. MP4 to GIF (Implemented in converter.rs)
- [x] 52. MP3 to WAV (Via FFmpeg)
- [x] 53. WAV to MP3 (Via FFmpeg)
- [x] 54. FLAC to MP3 (Via FFmpeg)
- [x] 55. AAC to MP3 (Via FFmpeg)
- [x] 56. OGG to WAV (Via FFmpeg)
- [x] 57. M4A to MP3 (Via FFmpeg)
- [x] 58. Extract audio track from MP4 to FLAC (Implemented via FFmpeg)
- [x] 59. Video mute (strip audio stream - Implemented via strip_metadata/settings)
- [x] 60. Subtitle extraction (SRT from MKV) (Implemented via FFmpeg)

## UI & UX
- [x] Implement human-friendly error mapping (errors.rs).
- [x] Implement intelligent pre-flight validation (converter.rs).
- [x] Add engine health diagnostic dashboard (DiagnosticModal.tsx).
- [x] Add 'Limited Engine Mode' for missing sidecar resilience.
- [ ] Audit UI responsiveness under load.
- [x] Improve error reporting in the UI.

## Testing
- [x] Establish Rust integration test suite (Added to converter.rs).
- [ ] Establish Vitest/React Testing Library setup for frontend.
