use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryStatus {
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
    pub path_or_sidecar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarHealthReport {
    pub ffmpeg: BinaryStatus,
    pub pandoc: BinaryStatus,
    pub imagemagick: BinaryStatus,
    pub pdftotext: BinaryStatus,
    pub all_ready: bool,
}

pub fn probe_binary(cmd_name: &str, version_arg: &str) -> BinaryStatus {
    // Prefer bundled binaries under src-tauri/binaries or current exe dir
    use std::env;
    use std::path::PathBuf;

    let base = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p2| p2.to_path_buf()))
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Try platform-specific folder
    let os = env::consts::OS;
    let arch = env::consts::ARCH;
    let platform = format!("{}-{}", os, arch);

    let mut candidate = base.join("binaries").join(&platform).join(cmd_name);
    if cfg!(windows) {
        candidate.set_extension("exe");
    }
    if candidate.exists() {
        let output = Command::new(candidate).arg(version_arg).output();
        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let first_line = stdout.lines().next().unwrap_or("Available").to_string();
                return BinaryStatus {
                    name: cmd_name.to_string(),
                    available: true,
                    version: Some(first_line),
                    path_or_sidecar: "bundled".to_string(),
                };
            }
        }
    }

    // During development, Tauri sidecars live next to this Cargo manifest with a
    // target-triple suffix (for example, ffmpeg-x86_64-unknown-linux-gnu).
    let target_suffix = if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        Some("x86_64-unknown-linux-gnu")
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        Some("aarch64-apple-darwin")
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        Some("x86_64-apple-darwin")
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        Some("x86_64-pc-windows-msvc.exe")
    } else {
        None
    };

    if let Some(suffix) = target_suffix {
        let candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(format!("{cmd_name}-{suffix}"));
        if candidate.exists() {
            let output = Command::new(candidate).arg(version_arg).output();
            if let Ok(out) = output {
                if out.status.success() {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let first_line = stdout.lines().next().unwrap_or("Available").to_string();
                    return BinaryStatus {
                        name: cmd_name.to_string(),
                        available: true,
                        version: Some(first_line),
                        path_or_sidecar: "bundled".to_string(),
                    };
                }
            }
        }
    }

    // Try generic binaries/<cmd_name>
    let mut cand2 = base.join("binaries").join(cmd_name);
    if cfg!(windows) {
        cand2.set_extension("exe");
    }
    if cand2.exists() {
        let output = Command::new(cand2).arg(version_arg).output();
        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let first_line = stdout.lines().next().unwrap_or("Available").to_string();
                return BinaryStatus {
                    name: cmd_name.to_string(),
                    available: true,
                    version: Some(first_line),
                    path_or_sidecar: "bundled".to_string(),
                };
            }
        }
    }

    // Fallback to system PATH
    let output = Command::new(cmd_name).arg(version_arg).output();
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let first_line = stdout.lines().next().unwrap_or("Available").to_string();
            BinaryStatus {
                name: cmd_name.to_string(),
                available: true,
                version: Some(first_line),
                path_or_sidecar: "system_path".to_string(),
            }
        }
        _ => BinaryStatus {
            name: cmd_name.to_string(),
            available: false,
            version: None,
            path_or_sidecar: "not_found".to_string(),
        },
    }
}

pub fn check_sidecar_health() -> SidecarHealthReport {
    let ffmpeg = probe_binary("ffmpeg", "-version");
    let pandoc = probe_binary("pandoc", "--version");
    let imagemagick = probe_binary("magick", "--version");
    let pdftotext = probe_binary("pdftotext", "-v");

    let all_ready =
        ffmpeg.available && pandoc.available && imagemagick.available && pdftotext.available;

    SidecarHealthReport {
        ffmpeg,
        pandoc,
        imagemagick,
        pdftotext,
        all_ready,
    }
}
