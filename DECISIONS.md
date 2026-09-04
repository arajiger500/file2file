# Architectural Decisions

## 2026-09-04: Engineering Memory Setup
- Established persistent memory files (GOAL.md, PROGRESSION.md, DECISIONS.md, FAILURES.md, TODO.md) at the root level to track autonomous development.

## 2026-09-04: Unified Sidecar Management
- Decided to move all external binaries (`magick`, `pdftotext` in addition to `ffmpeg`, `pandoc`) to Tauri sidecars for better portability and consistency.
- Updated backend logic to always prefer sidecars over system path for conversions.

## 2026-09-04: Frontend Service Refactoring
- Extracted browser-side conversion logic into a separate `universal-engine.ts` to keep `api.ts` focused on IPC and high-level routing.
- This allows for easier testing and expansion of browser-based fallback conversions.
