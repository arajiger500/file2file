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
    pub svgo: BinaryStatus,
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
    if cfg!(windows) { candidate.set_extension("exe"); }
    if candidate.exists() {
        let output = Command::new(candidate).arg(version_arg).output();
        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let first_line = stdout.lines().next().unwrap_or("Available").to_string();
                return BinaryStatus { name: cmd_name.to_string(), available: true, version: Some(first_line), path_or_sidecar: "bundled".to_string() };
            }
        }
    }

    // Try generic binaries/<cmd_name>
    let mut cand2 = base.join("binaries").join(cmd_name);
    if cfg!(windows) { cand2.set_extension("exe"); }
    if cand2.exists() {
        let output = Command::new(cand2).arg(version_arg).output();
        if let Ok(out) = output {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let first_line = stdout.lines().next().unwrap_or("Available").to_string();
                return BinaryStatus { name: cmd_name.to_string(), available: true, version: Some(first_line), path_or_sidecar: "bundled".to_string() };
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
    let svgo = probe_binary("svgo", "--version");

    let all_ready = ffmpeg.available && pandoc.available && svgo.available;

    SidecarHealthReport {
        ffmpeg,
        pandoc,
        svgo,
        all_ready,
    }
}
