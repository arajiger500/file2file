# File2File - Universal Local File Converter

A high-performance, **100% offline and private** universal desktop file converter built with **Tauri 2.0 (Rust backend)** and **React + TypeScript + Tailwind CSS (frontend)**.

---

## 🚀 Key Features

- **🔒 100% Local & Offline Processing:** No cloud uploads, telemetry, or external API requirements. All transcoding runs locally using high-performance sidecar binaries (`FFmpeg`, `Pandoc`, `SVGO`).
- **⚡ Hardware Auto-Detection & Acceleration:** Probes available GPU hardware encoders on startup:
  - **NVIDIA:** NVENC (`h264_nvenc`, `hevc_nvenc`)
  - **AMD / Linux:** VA-API / AMF (`h264_vaapi`, `h264_amf`)
  - **Intel:** QuickSync (`h264_qsv`)
  - **Apple Silicon:** VideoToolbox (`h264_videotoolbox`)
  - **CPU Fallback:** Multi-threaded software encoding (`libx264`, `libx265`) auto-scaled to available CPU cores.
- **📂 Intuitive Drag & Drop Workspace:** Supports batch file selection and whole directory drops.
- **🎯 100+ Conversion Pathways:** Displays valid conversion targets with pro-grade comparison notes (e.g., explaining WebP vs. AVIF, structural PDF to Word extraction, and lossless audio targets).
- **⚙️ Advanced Settings Drawer:** Rate control (CRF slider 0–51), resolution downscaling (4K, 1080p, 720p, SD), audio bitrates, and one-click privacy metadata stripping.
- **⚡ Quick Workflows & Session Logs:** Pre-configured shortcuts (e.g., PNG → WebP, Video → MP4/GIF, Markdown → PDF) and session history showing compression savings.

---

## 📁 Project Architecture & Directory Structure

```
file2file/
├── src-tauri/                       # Rust Tauri 2.0 Backend
│   ├── binaries/                    # Target-specific sidecar binaries
│   │   └── README.md
│   ├── capabilities/
│   │   └── default.json             # Tauri 2.0 permissions & sidecar capability
│   ├── src/
│   │   ├── converter.rs             # Transcoding orchestrator & CLI command builder
│   │   ├── formats.rs               # Format registry, categories, & contextual recommendations
│   │   ├── hardware.rs              # GPU hardware encoder auto-detection & CPU fallback
│   │   ├── sidecar.rs               # Native binary probing & health check
│   │   ├── lib.rs                   # Tauri application entry & IPC command handlers
│   │   └── main.rs                  # Native application entry point
│   ├── build.rs
│   ├── Cargo.toml                   # Rust dependencies (Tauri 2, tokio, serde, num_cpus)
│   └── tauri.conf.json              # Tauri 2.0 window & bundle configuration
├── src/                             # React + TypeScript Frontend
│   ├── components/
│   │   ├── AdvancedSettingsDrawer.tsx # Drawer for CRF, resolution, HW toggles, privacy
│   │   ├── DropZone.tsx             # File & folder drag-and-drop workspace
│   │   ├── FormatSelector.tsx       # Contextual target formats & comparison badges
│   │   ├── Header.tsx               # Hardware status pill & navigation
│   │   ├── HistoryPanel.tsx         # Conversion logs, duration & size savings
│   │   └── QuickConverters.tsx      # One-click workflow presets
│   ├── services/
│   │   └── api.ts                   # Tauri IPC bridge with browser dev mock fallback
│   ├── types/
│   │   └── index.ts                 # TypeScript type definitions
│   ├── App.tsx                      # Root application layout & state
│   ├── index.css                    # Tailwind CSS directives & custom styling
│   └── main.tsx                     # React DOM root
├── package.json                     # Frontend dependencies & scripts
├── tailwind.config.js               # Tailwind CSS theme extension
├── tsconfig.json                    # TypeScript configuration
└── vite.config.ts                   # Vite + Tauri development server config
```

---

## 🛠️ Setup & Development Commands

### Prerequisites
- **Node.js**: v18+ and `npm`
- **Rust**: `rustc` and `cargo` (via [rustup.rs](https://rustup.rs/))
- **Sidecar Binaries or System Packages**: `ffmpeg`, `pandoc`, `magick` (ImageMagick), `svgo`

### 1. Install Dependencies
```bash
# Install frontend packages
npm install
```

### 2. Run in Development Mode
```bash
# Option A: Run complete Tauri Desktop application with live reload
npm run tauri dev

# Option B: Run frontend only in standard browser (with mock IPC responses)
npm run dev
```

### 3. Production Build
```bash
# Build desktop native installers (.deb, .rpm, .dmg, .msi, .exe)
npm run tauri build
```

---

## 📦 Native Sidecar Placement

For standalone, self-contained desktop builds, download target-specific sidecar binaries into `src-tauri/binaries/` following Tauri's target-triple naming convention:

| Utility | Linux Triple Example | Windows Triple Example | macOS Triple Example |
| :--- | :--- | :--- | :--- |
| **FFmpeg** | `ffmpeg-x86_64-unknown-linux-gnu` | `ffmpeg-x86_64-pc-windows-msvc.exe` | `ffmpeg-aarch64-apple-darwin` |
| **Pandoc** | `pandoc-x86_64-unknown-linux-gnu` | `pandoc-x86_64-pc-windows-msvc.exe` | `pandoc-aarch64-apple-darwin` |
| **SVGO** | `svgo-x86_64-unknown-linux-gnu` | `svgo-x86_64-pc-windows-msvc.exe` | `svgo-aarch64-apple-darwin` |

*Note: During local development, File2File automatically resolves standard system-installed binaries from your `PATH` if bundle sidecars are not yet present.*
