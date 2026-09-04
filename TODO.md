# Priority TODO List

## Core Infrastructure
- [x] Verify build health (`npm run build`, `cargo check`).
- [x] Audit current hardware acceleration detection (Updated to use sidecar).
- [ ] Audit security of file path handling and shell execution.
- [x] Ensure all binaries (magick, pdftotext) are handled as sidecars.

## Conversion Implementation
### Phase 2: Document Conversions (1 to 15)
- [ ] 1. PDF to DOCX
- [ ] 2. DOCX to PDF
- [ ] 3. PDF to TXT
- [ ] 4. PDF to HTML
- [ ] 5. HTML to PDF
- [ ] 6. Markdown to PDF
- [ ] 7. Markdown to HTML
- [ ] 8. EPUB to PDF
- [ ] 9. EPUB to TXT
- [ ] 10. MOBI to EPUB
- [ ] 11. RTF to DOCX
- [ ] 12. DOCX to TXT
- [ ] 13. ODT to PDF
- [ ] 14. ODT to DOCX
- [x] 15. CSV to JSON (Implemented via Rust native)

### Phase 3: Data, Spreadsheet & Structured Conversions (16 to 30)
- [ ] 16. CSV to XLSX
- [ ] 17. XLSX to CSV
- [ ] 18. XLSX to JSON
- [x] 19. JSON to CSV
- [x] 20. JSON to YAML
- [x] 21. YAML to JSON
- [ ] 22. XML to JSON
- [ ] 23. JSON to XML
- [x] 24. TOML to JSON
- [x] 25. JSON to TOML
- [ ] 26. SQLite DB to JSON dump
- [ ] 27. CSV to SQL Insert statements
- [ ] 28. Log file to structured JSON
- [ ] 29. BibTeX to JSON
- [ ] 30. ICS Calendar to JSON

### Phase 4: Image & Raster Graphics Conversions (31 to 45)
- [x] 31. PNG to JPEG (Via ImageMagick)
- [x] 32. JPEG to PNG (Via ImageMagick)
- [x] 33. PNG to WebP (Via ImageMagick)
- [x] 34. WebP to PNG (Via ImageMagick)
- [x] 35. PNG to AVIF (Via ImageMagick)
- [x] 36. AVIF to PNG (Via ImageMagick)
- [ ] 37. SVG to PNG (rasterization)
- [ ] 38. HEIC to JPEG
- [ ] 39. TIFF to PNG
- [ ] 40. BMP to JPEG
- [ ] 41. GIF to WebP animation
- [ ] 42. PNG to ICO (multi-resolution icon generation)
- [ ] 43. RAW camera format to JPEG preview
- [ ] 44. PSD thumbnail extraction to PNG
- [ ] 45. SVG to optimized minimal SVG

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
- [ ] 58. Extract audio track from MP4 to FLAC
- [x] 59. Video mute (strip audio stream - Implemented via strip_metadata/settings)
- [ ] 60. Subtitle extraction (SRT from MKV)

## UI & UX
- [ ] Audit UI responsiveness under load.
- [ ] Improve error reporting in the UI.

## Testing
- [ ] Establish Rust integration test suite.
- [ ] Establish Vitest/React Testing Library setup for frontend.
