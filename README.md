# File2File

> **Universal local file conversion — private by design.**
>
> Convert media, documents, images, structured data, and archives without sending your files to a cloud service.

[![Tauri 2](https://img.shields.io/badge/Tauri-2.0-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/backend-Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/frontend-React-61DAFB?logo=react&logoColor=111)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.7-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-3.4-06B6D4?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)

File2File is a Tauri 2 desktop application with a React + TypeScript UI and a Rust conversion core. Its architecture is designed around **local processing first**: desktop conversions use local binaries/sidecars, while the browser development mode provides a lightweight fallback engine for supported transformations.

## ✨ What File2File Can Do

### Local conversion core

The desktop app routes conversions through the appropriate local engine:

| Content type | Main engine | Examples |
|---|---|---|
| Video & audio | **FFmpeg** | MP4, MKV, MOV, AVI, WebM, MP3, WAV, FLAC, AAC, OGG, M4A, Opus |
| Images & graphics | **ImageMagick** | PNG, JPEG, WebP, AVIF, TIFF, BMP, HEIC, PSD, SVG-related workflows |
| Documents | **Pandoc + Poppler** | PDF, DOCX, Markdown, HTML, TXT, EPUB, RTF, ODT |
| Structured data | **Rust-native** | CSV, JSON, YAML, TOML, XML, XLSX, SQL, SQLite, BibTeX, ICS, logs |
| Archives | **Rust-native** | ZIP creation and extraction |

The project currently exposes a large catalog of conversion paths and groups formats by video, audio, image, document, vector, data, and archive categories. The UI also surfaces format-specific metadata such as lossless status, recommended targets, engine/core, pros, and cons.

### 🧠 Smart format recommendations

After importing a file, File2File can generate a short list of recommended output formats based on the input type. There is also a dedicated format browser with search and category filters for exploring compatible targets.

### 📦 Batch conversion

Add multiple files to a queue and process them as a batch. Desktop drag-and-drop accepts individual files **and entire directories**; directories are recursively scanned before being added to the queue.

### ⚙️ Advanced conversion controls

The settings drawer exposes:

- **CRF / rate control:** 0–51
- **Resolution targeting:** Original, 4K UHD, 1080p, 720p, or SD
- **Hardware acceleration:** enabled/disabled when supported
- **Encoder selection:** automatic selection or a detected encoder profile
- **Metadata stripping:** remove common privacy-sensitive metadata such as GPS, camera, author, and timestamp fields where the selected conversion supports it
- **Audio bitrate:** 64k–320k presets

### 🚀 Hardware-aware encoding

At startup, the Rust backend probes the available FFmpeg encoder capabilities and platform hints. Depending on the machine and installed FFmpeg build, File2File can expose hardware encoders such as:

- NVIDIA **NVENC** (`h264_nvenc`, `hevc_nvenc`)
- Intel **Quick Sync** (`h264_qsv`)
- AMD / Linux **VA-API / AMF** (`h264_vaapi`)
- Apple **VideoToolbox** (`h264_videotoolbox`)
- CPU fallback using **libx264** / **libx265**

The application also reports the detected CPU core count and uses it to size its batch worker pool.

### ✅ Pre-flight validation

Before a batch starts, media files can be inspected with `ffprobe`. Validation checks that inputs exist and, for media jobs, verifies stream availability such as whether an audio track exists for an audio-only target. Warnings and friendly error messages are surfaced in the UI instead of exposing raw process failures where possible.

### 🩺 Engine diagnostics

The built-in **Engine Health** panel checks the availability and version of the conversion cores:

- FFmpeg
- FFprobe
- Pandoc
- ImageMagick (`magick`)
- Poppler `pdftotext`

File2File can operate in a **limited-capability mode** when some tools are unavailable, while keeping the diagnostics visible so missing dependencies are easy to identify.

### 📜 Session history

Completed jobs are recorded in the current session with:

- output filename/path
- success or failure state
- elapsed conversion time
- original vs. converted size
- estimated percentage size change
- conversion error details when a job fails

There is also a **Clear Logs** action for the session history.

## ⚡ Quick Workflows

The dashboard exposes preset workflows for common jobs, including examples such as:

- PDF → editable Word
- Image → WebP / AVIF
- Video → MP4
- Video → MP3 / FLAC
- Video → animated WebP
- Image → SVG/vector-oriented workflow

Presets are intentionally presented as shortcuts into the same conversion pipeline rather than as separate conversion systems.

## 🌐 Browser / Development Fallback

Running the frontend with Vite does not require the desktop shell. Outside Tauri, `api.ts` routes calls to a browser-side `universal-engine.ts` implementation.

That fallback currently supports real in-browser transformations for selected workflows such as:

- PDF → DOCX (text extraction into a generated DOCX package)
- CSV ↔ JSON
- Image → PNG / JPEG / WebP where the browser codec is available
- Image → PDF
- Text / Markdown / HTML → PDF or text/HTML outputs

For conversions that require the native desktop toolchain, the browser engine returns a clear message directing the user to the desktop application.

> **Important:** browser fallback is a development/resilience feature, not a promise that every desktop conversion is available in a normal web browser.

## 🔒 Privacy & Security

File2File is designed around a local-first workflow:

- Conversion requests are handled locally by the desktop application and its conversion tools.
- Desktop tooling prefers bundled Tauri sidecars, then checks the application's local `bin/` directory, with system `PATH` as a fallback.
- Shell commands are constructed with explicit argument arrays rather than shell-string interpolation.
- Target format values are sanitized before they are used in output filenames.
- File paths are validated before conversion and path handling avoids unsafe `unwrap()` assumptions in the Rust conversion code.
- PDF conversion uses a temporary text file and removes it after the operation completes.

**Privacy note:** “local/offline” depends on the desktop build and the binaries available to it. The application itself is designed not to upload conversion inputs, but the browser fallback can load its PDF.js worker from a CDN when that workflow is used.

## 🏗️ Architecture

```text
file2file/
├── src/                              # React + TypeScript UI
│   ├── components/
│   │   ├── AdvancedSettingsDrawer.tsx
│   │   ├── ConversionStudio.tsx
│   │   ├── DiagnosticModal.tsx
│   │   ├── DropZone.tsx
│   │   ├── FormatSelectionPage.tsx
│   │   ├── Header.tsx
│   │   ├── HistoryPanel.tsx
│   │   └── QuickConverters.tsx
│   ├── services/
│   │   ├── api.ts                    # Tauri IPC + browser fallback router
│   │   └── universal-engine.ts       # Selected browser-side conversions
│   ├── types/
│   │   └── index.ts
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
│
├── src-tauri/                        # Rust + Tauri 2 backend
│   ├── binaries/                     # Optional bundled sidecars
│   ├── capabilities/
│   └── src/
│       ├── converter.rs              # Validation, routing & conversion execution
│       ├── formats.rs                # Format catalog & smart recommendations
│       ├── hardware.rs               # Encoder/GPU capability detection
│       ├── sidecar.rs                # Engine probing & command resolution
│       ├── errors.rs                 # Human-friendly error mapping
│       ├── lib.rs                    # Tauri commands & application setup
│       └── main.rs
│
├── GOAL.md                           # Project objective
├── PROGRESSION.md                    # Development journal
├── DECISIONS.md                      # Architectural decisions
├── FAILURES.md                       # Known failures / incidents
├── TODO.md                           # Implementation checklist
├── package.json
├── tailwind.config.js
├── tsconfig.json
├── vite.config.ts
└── LICENSE
```

## 🔧 How the Desktop Pipeline Works

At a high level, a conversion follows this path:

```text
User drops files
      │
      ▼
File detection + category inference
      │
      ├── Smart suggestions
      └── Compatible format catalog
      │
      ▼
Pre-flight validation
      │
      ▼
Conversion Studio
      │
      ├── Rust-native data/archive conversion
      ├── Pandoc / Poppler document conversion
      ├── ImageMagick graphics conversion
      └── FFmpeg media conversion
                │
                ├── hardware encoder when available
                └── CPU fallback
      │
      ▼
Result + size/time metrics
      │
      ▼
Session history / export
```

For batches, the frontend starts multiple workers and uses the detected CPU core count as the concurrency limit. Each file is validated and then passed independently through the same backend conversion path.

## 📦 Requirements

### Desktop development

You need:

- **Node.js** 18+ and npm
- **Rust** + Cargo (recommended installation via [rustup](https://rustup.rs/))
- Tauri 2 development prerequisites for your operating system

For the desktop conversion toolchain, provide:

- `ffmpeg`
- `ffprobe`
- `pandoc`
- `magick` (ImageMagick)
- `pdftotext` (Poppler)

The application can use bundled sidecars instead of requiring all binaries to be globally installed.

## ▶️ Development

Install JavaScript dependencies:

```bash
npm install
```

Run the browser frontend:

```bash
npm run dev
```

Run the full Tauri desktop application with live reload:

```bash
npm run tauri dev
```

Build the frontend:

```bash
npm run build
```

Build the desktop bundles/installers:

```bash
npm run tauri build
```

The Tauri configuration is set up to bundle targets across Windows, macOS, and Linux, with NSIS/WiX support on Windows and DMG/AppImage/DEB-oriented packaging configuration for desktop distribution.

## 🧩 Sidecar Layout

Tauri expects external binaries to follow its target-triple naming convention inside `src-tauri/binaries/`.

Example layout:

```text
src-tauri/binaries/
├── ffmpeg-<target-triple>[.exe]
├── ffprobe-<target-triple>[.exe]
├── pandoc-<target-triple>[.exe]
├── magick-<target-triple>[.exe]
└── pdftotext-<target-triple>[.exe]
```

When a bundled binary is unavailable, the backend checks the app data `bin/` directory and then the normal system `PATH`.

## 🧪 Testing & Project Status

The Rust side includes unit/integration-oriented tests for core functionality. Recent development work has covered JSON/CSV edge cases, ZIP handling, path validation, sidecar management, and conversion error handling.

The project is under active development. Some formats and workflows are deliberately marked as incomplete or limited, and frontend Vitest/React Testing Library coverage is still planned.

For the living implementation checklist and current roadmap, see [`TODO.md`](./TODO.md). For implementation history, see [`PROGRESSION.md`](./PROGRESSION.md).

## 🗺️ Current Scope

The current codebase includes conversion support across these broad families:

- **Documents:** PDF, DOCX, Markdown, HTML, TXT, EPUB, RTF, ODT
- **Data:** CSV, JSON, YAML, TOML, XML, XLSX, SQL, SQLite, BibTeX, ICS, logs
- **Images / graphics:** PNG, JPEG, WebP, AVIF, BMP, TIFF, ICO, HEIC, TGA, PSD, SVG-related workflows
- **Media:** MP4, MKV, MOV, AVI, WebM, GIF plus common audio formats such as MP3, WAV, FLAC, AAC, OGG, M4A, and Opus
- **Archives:** ZIP creation/extraction and recursive folder-to-archive workflows

Exact compatibility is determined by the input/target pair and the engines available in the current desktop installation. The UI intentionally filters and recommends conversions instead of treating every possible pair as valid.

## 🤝 Contributing

Contributions are welcome. Before making a larger change, review the project memory files:

```text
GOAL.md
PROGRESSION.md
DECISIONS.md
FAILURES.md
TODO.md
```

When adding a new conversion path, update the format catalog, backend routing, validation/error handling, and relevant tests rather than adding a one-off UI shortcut.

## 📄 License

See [`LICENSE`](./LICENSE).

---

**File2File** — convert files locally, keep the pipeline understandable, and keep your data where it belongs: on your machine.
