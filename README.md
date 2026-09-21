# File2File

> **A local-first desktop file converter for Windows, macOS and Linux.**
>
> Convert documents, images, audio, video and structured data without uploading your files to a cloud service.

[![Version](https://img.shields.io/badge/version-0.2.1-informational)](https://github.com/arajiger500/file2file/releases)
[![Tauri 2](https://img.shields.io/badge/Tauri-2.x-24C8DB?logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/backend-Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/frontend-React-61DAFB?logo=react&logoColor=111)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/frontend-TypeScript-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)

## Download

Each tagged release targets:

| Platform | Artifacts |
|---|---|
| Windows x64 | NSIS installer + MSI (unsigned unless release signing is configured) |
| macOS Apple Silicon | DMG (unsigned/not notarized unless release signing is configured) |
| macOS Intel | DMG (unsigned/not notarized unless release signing is configured) |
| Linux x64 | AppImage + DEB |

[Open the latest releases](https://github.com/arajiger500/file2file/releases)

The release workflow builds each package on a native GitHub-hosted runner. The already-published v0.2.0 release predates this packaging contract; v0.2.1 is the first release prepared by the hardened workflow.

### Installation

- **Windows:** download the x64 setup executable or MSI and follow the installer prompts.
- **Linux:** install the x64 DEB on Debian-compatible systems, or make the AppImage executable and run it.
- **macOS:** choose the DMG matching Apple Silicon or Intel, then drag File2File to Applications.

Packages remain unsigned unless release signing is configured. Windows SmartScreen or macOS Gatekeeper may therefore require explicit confirmation; verify the published SHA-256 checksums before installing.

## What it does

File2File provides one conversion workflow for several file families:

- **Documents:** PDF, DOCX, HTML, Markdown, TXT, EPUB, RTF, ODT
- **Data:** CSV, JSON, YAML, TOML, XML, XLSX, SQL, SQLite, BibTeX, ICS and logs
- **Images:** PNG, JPEG, WebP, BMP, TIFF, ICO, TGA, PSD and SVG
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

1. a real executable beside the application, when present
2. application data `bin/` directory
3. system `PATH`

File2File does not bundle converter engines. The desktop application can start without them, but conversion categories that depend on external engines require those engines to be installed. Engine Health distinguishes a missing executable from one that was found but failed its bounded identity/version probe.

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

The Engine Health panel shows which engines File2File can currently see. Restart the application or use the refresh control after changing `PATH` or the application-data `bin/` directory.

## Features

### Batch conversion

Drop multiple files for a bounded batch. Selecting or dropping a directory creates one folder-to-ZIP job; links, junctions and special files are rejected.

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

Media inputs are probed before conversion. Jobs stage output in a private workspace on the destination filesystem and publish it without following output symlinks. Existing files are preserved unless overwrite is explicitly selected; overwrite atomically replaces regular files. Cancellation covers queued work, child-process trees and final publication.

### Safety limits

- batches and selected input lists: 256 entries
- structured-data input: 32 MiB; expanded XLSX: 128 MiB
- archive conversion: 10,000 entries, 64 path levels and 5 GiB expanded data
- external process output captured for diagnostics: 4 MiB per stream
- browser fallback: 64 MiB input and 100 megapixels for image transcoding

Archives reject traversal, absolute/drive/UNC paths, links, special files, duplicate names and case-insensitive collisions. Conversion tools still parse hostile third-party formats, so keep FFmpeg, ImageMagick, Pandoc and Poppler patched.

### Fidelity limits

- PDF conversion is text/layout extraction; complex layout, fonts, forms and images may shift or be lost.
- Document conversion is semantic rather than pixel-perfect and may change styles or metadata.
- Structured conversions use the first XLSX worksheet. CSV has no native types; numeric-looking values can change type, while identifiers with leading zeroes remain strings. CSV output prefixes formula-like cells with an apostrophe to prevent spreadsheet formula execution.
- Image conversion to JPEG flattens transparency onto white. Media conversion is lossy unless the selected codec/format is lossless.
- ZIP/TAR conversion intentionally omits links and platform-specific special files.

### Browser fallback

Running the Vite frontend outside Tauri enables a limited browser-only fallback for supported lightweight transformations.

It is **not** a replacement for the desktop conversion pipeline and does not provide native FFmpeg, ImageMagick, Pandoc or GPU functionality.

## Privacy

Desktop conversion is local. File2File does not require a cloud conversion service.

The browser development fallback is intentionally separate from the desktop engine stack. Its PDF.js worker is packaged by the frontend build; it does not provide native FFmpeg, ImageMagick, Pandoc or GPU functionality.

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

```bash
npm run tauri build
```

The desktop build also requires the platform packages listed by the [Tauri prerequisites guide](https://v2.tauri.app/start/prerequisites/).

## Testing

Run the deterministic frontend and Rust checks:

```bash
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo test --locked --manifest-path src-tauri/Cargo.toml --lib --test native_safety
cargo test --locked --manifest-path src-tauri/Cargo.toml --tests --no-run
```

The opt-in fidelity tests under `src-tauri/tests/` generate their own fixtures and are ignored by default because they require specific local converters. For example:

```bash
cargo test --locked --manifest-path src-tauri/Cargo.toml --test torture_tests -- --ignored
```

CI runs:

- frontend build/typecheck
- Rust formatting
- Rust library tests
- native desktop builds

Release CI additionally produces the platform installers/packages.

## Release process

Keep the npm, Cargo and Tauri versions identical, merge the version commit, then push a new tag matching that version:

```bash
git tag v0.2.1
git push origin v0.2.1
```

The release workflow builds the following native targets:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

All four native builds must succeed before the workflow creates or updates a draft GitHub release. The draft contains six installers and a SHA-256 checksum file. A maintainer must install and smoke-test every target, sign/notarize where applicable, and then publish the draft. Never reuse or move a published version tag.

## Troubleshooting

- **Engine is missing:** install the named converter and ensure its directory is on `PATH`, or place the executable in the application-data `bin/` directory, then restart or refresh Engine Health.
- **Engine is unusable:** run that executable's version command in a terminal and repair or replace the reported binary. File2File will not bypass a broken higher-priority app-local engine.
- **A conversion pair is absent:** it is not enabled by the format registry; changing the filename extension does not make an unsupported conversion valid.
- **Output already exists:** choose auto-rename, skip, or explicit overwrite. Extracted directories are never overwritten.
- **macOS or Windows blocks installation:** current packages may be unsigned. Confirm the checksum and use the platform's documented manual approval flow, or build from source.

## License

See [LICENSE](./LICENSE).

---

**File2File v0.2.1** — local conversion, explicit engines, predictable packaging.
