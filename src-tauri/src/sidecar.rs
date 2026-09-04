use serde::{Deserialize, Serialize};
use tauri_plugin_shell::ShellExt;

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
    pub ffprobe: BinaryStatus,
    pub pandoc: BinaryStatus,
    pub imagemagick: BinaryStatus,
    pub pdftotext: BinaryStatus,
    pub all_ready: bool,
}

fn parse_version(output: &str, cmd: &str) -> String {
    let first_line = output.lines().next().unwrap_or("Available");
    match cmd {
        "ffmpeg" | "ffprobe" => {
            first_line
                .split("version ")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or(first_line)
                .to_string()
        }
        "magick" => {
            first_line
                .split("ImageMagick ")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or(first_line)
                .to_string()
        }
        "pandoc" => {
            first_line
                .split_whitespace()
                .nth(1)
                .unwrap_or(first_line)
                .to_string()
        }
        _ => first_line.to_string(),
    }
}

pub async fn probe_sidecar(
    app: &tauri::AppHandle,
    cmd_name: &str,
    version_arg: &str,
) -> BinaryStatus {
    match app.shell().sidecar(cmd_name) {
        Ok(sidecar) => {
            let output = sidecar.arg(version_arg).output().await;
            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let version = parse_version(&stdout, cmd_name);
                    BinaryStatus {
                        name: cmd_name.to_string(),
                        available: true,
                        version: Some(version),
                        path_or_sidecar: "bundled".to_string(),
                    }
                }
                _ => BinaryStatus {
                    name: cmd_name.to_string(),
                    available: false,
                    version: None,
                    path_or_sidecar: "error".to_string(),
                },
            }
        }
        Err(_) => {
            // Fallback to system PATH for health report
            let output = std::process::Command::new(cmd_name).arg(version_arg).output();
            match output {
                Ok(out) if out.status.success() => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let version = parse_version(&stdout, cmd_name);
                    BinaryStatus {
                        name: cmd_name.to_string(),
                        available: true,
                        version: Some(version),
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
    }
}

pub async fn check_sidecar_health(app: &tauri::AppHandle) -> SidecarHealthReport {
    let ffmpeg = probe_sidecar(app, "ffmpeg", "-version").await;
    let ffprobe = probe_sidecar(app, "ffprobe", "-version").await;
    let pandoc = probe_sidecar(app, "pandoc", "--version").await;
    let imagemagick = probe_sidecar(app, "magick", "--version").await;
    let pdftotext = probe_sidecar(app, "pdftotext", "-v").await;

    let all_ready =
        ffmpeg.available && ffprobe.available && pandoc.available && imagemagick.available && pdftotext.available;

    SidecarHealthReport {
        ffmpeg,
        ffprobe,
        pandoc,
        imagemagick,
        pdftotext,
        all_ready,
    }
}
