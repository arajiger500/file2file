# File2File v0.2.0

> **A local-first desktop file converter for Windows, macOS and Linux.**
>
> Convert documents, images, audio, video and structured data without uploading your files to a cloud service.

[![Version](https://img.shields.io/badge/version-0.2.0-informational)](https://github.com/arajiger500/file2file/releases/tag/v0.2.0)
[![Tauri 2](https://img.shields.io/badge/Tauri-2.x-24C8DB?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/backend-Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/frontend-React-61DAFB?logo=react&logoColor=111)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/frontend-TypeScript-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)

## Download

Release **v0.2.0** is built for:

| Platform | Artifacts |
|---|---|
| Windows x64 | NSIS installer + MSI |
| macOS Apple Silicon | DMG |
| macOS Intel | DMG |
| Linux x64 | AppImage + DEB |

[Open the latest releases](https://github.com/arajiger500/file2file/releases)

The release workflow runs independently on each native runner, so each package is built for its real CPU architecture.

## What it does

File2File provides one conversion workflow for several file families:

- **Documents:** PDF, DOCX, HTML, Markdown, TXT, EPUB, RTF, ODT
- **Data:** CSV, JSON, YAML, TOML, XML, XLSX, SQL, SQLite, BibTeX, ICS and logs
- **Images:** PNG, JPEG, WebP, AVIF, BMP, TIFF, ICO, HEIC, TGA, PSD and supported vector inputs
- **Media:** MP4, MKV, MOV, AVI, WebM, GIF, MP3, WAV, FLAC, AAC, OGG, M4A and Opus
- **Archives:** ZIP creation/extraction and folder archiving

Actual availability is determined by the input/target pair and the conversion engines installed on the machine.

## Desktop architecture

```text
React + TypeScript UI
        │
        ▼
Tauri 2 IPC
        │
        ▼
Rust conversion core
   ┌────┼───────────────┐
   │    │               │
FFmpeg ImageMagick   Pandoc/Poppler
   │    │               │
   └────┼───────────────┘
        │
 Rust-native data/archive conversion
```

The backend resolves conversion engines in this order:

1. app-local/bundled engine when available
2. application data `bin/` directory
3. system `PATH`

This release intentionally does **not** fake bundled converter binaries. The desktop application can start without them, but conversion categories that depend on external engines require those engines to be installed.

## Engine requirements

### Windows

Install:

- FFmpeg + FFprobe
- Pandoc
- ImageMagick
- Poppler utilities (`pdftotext`, `pdftohtml`)

Make sure the executables are available on `PATH`.

### macOS

Install the same engine set with your preferred package manager, or place the executables in the application's local `bin/` directory.

### Linux

The DEB package declares the main runtime packages it needs. AppImage users should install:

- FFmpeg + FFprobe
- Pandoc
- ImageMagick
- Poppler utilities

The Engine Health panel shows which engines File2File can currently see.

## Features

### Batch conversion

Drop multiple files or a directory. Directories are scanned recursively and jobs are processed through a bounded queue.

### Smart format selection

Compatible targets and smart suggestions are derived from the format registry rather than exposing arbitrary extension combinations.

### Advanced controls

Depending on the conversion:

- CRF / rate control
- target resolution
- hardware encoder selection
- metadata stripping
- audio bitrate
- output collision policy
- retry/cancel handling

### Validation and failure handling

Media inputs can be probed before conversion. Jobs verify their final output before reporting success, use isolated temporary workspaces, and expose human-readable errors.

### Browser fallback

Running the Vite frontend outside Tauri enables a limited browser-only fallback for supported lightweight transformations.

It is **not** a replacement for the desktop conversion pipeline and does not provide native FFmpeg, ImageMagick, Pandoc or GPU functionality.

## Privacy

Desktop conversion is local. File2File does not require a cloud conversion service.

The browser development fallback is intentionally separate from the desktop engine stack; it can use browser resources such as PDF.js and should not be treated as an equivalent to the fully offline desktop environment.

## Development

Requirements:

- Node.js + npm
- Rust + Cargo
- Tauri 2 prerequisites for your OS

Install dependencies:

```bash
npm ci
```

Run the browser UI:

```bash
npm run dev
```

Run the desktop app:

```bash
npm run tauri dev
```

Build the frontend:

```bash
npm run build
```

Build the desktop package locally:

```npm run tauri build
```

## Testing

The project contains Rust tests and conversion-oriented integration tests.

CI runs:

- frontend build/typecheck
- Rust formatting
- Rust library tests
- native desktop builds

Release CI additionally produces the platform installers/packages.

## Release process

Create a version commit, then push a tag such as:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The release workflow builds the following native targets:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

The GitHub Actions release is created from that tag and publishes the native installers/packages.

## Project files

```text
src/                React + TypeScript UI
src-tauri/          Tauri + Rust backend
GOAL.md             project objectives
TODO.md             implementation checklist
PROGRESSION.md      engineering history
DECISIONS.md        architecture decisions
FAILURES.md         failure log
```

## License

See [LICENSE](./LICENSE).

---

**File2File v0.2.0** — local conversion, explicit engines, predictable packaging.
