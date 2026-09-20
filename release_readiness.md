# Release Readiness — v0.2.0

This file previously contained a bulk conversion matrix whose fixtures included placeholder/corrupt files. Its old 440-case score is therefore **historical data, not a valid production gate**.

## Current release gate

The authoritative checks for v0.2.0 are the GitHub Actions workflows:

- `.github/workflows/ci.yml`: native verification on Linux x64, Windows x64, macOS arm64 and macOS x64.
- `.github/workflows/publish.yml`: release packaging for those same native targets.
- Frontend typecheck/build must pass.
- Rust formatting and library tests must pass.
- The Tauri release build must complete successfully for every matrix entry.

## v0.2.0 packaging targets

| OS | Architecture | Package |
|---|---|---|
| Linux | x64 | AppImage + DEB |
| Windows | x64 | NSIS + MSI |
| macOS | arm64 | DMG |
| macOS | x64 | DMG |

## Important limitation

The release packages do not embed the large third-party conversion engines. File2File discovers FFmpeg/FFprobe, Pandoc, ImageMagick and Poppler locally.

That is intentional: the release pipeline must never ship fake/dummy executables or silently claim portable engine support that has not been verified.

Before calling a release fully production-ready, verify at least one representative real conversion on each target OS after installation and confirm the Engine Health panel reports the expected engines.
