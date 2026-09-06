use serde::{Deserialize, Serialize};
use tauri_plugin_shell::ShellExt;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryStatus {
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
    pub path_or_sidecar: String,
    pub absolute_path: Option<String>,
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
        "pdftotext" => {
            first_line
                .split("version ")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or(first_line)
                .to_string()
        }
        _ => first_line.to_string(),
    }
}

pub async fn probe_sidecar<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    cmd_name: &str,
    version_arg: &str,
) -> BinaryStatus {
    // 1. Try bundled sidecar
    if let Ok(sidecar) = app.shell().sidecar(cmd_name) {
        let output = sidecar.arg(version_arg).output().await;
        match output {
            Ok(out) if out.status.success() || !out.stderr.is_empty() => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let combined = format!("{}{}", stdout, stderr);
                let version = parse_version(&combined, cmd_name);

                if version != "Available" || out.status.success() {
                    return BinaryStatus {
                        name: cmd_name.to_string(),
                        available: true,
                        version: Some(version),
                        path_or_sidecar: "bundled".to_string(),
                        absolute_path: None,
                    };
                }
            }
            _ => {}
        }
    }

    // 2. Try App Data Directory fallback
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let bin_path = app_data_dir.join("bin").join(if cfg!(windows) {
            format!("{}.exe", cmd_name)
        } else {
            cmd_name.to_string()
        });

        if bin_path.exists() {
            let output = std::process::Command::new(&bin_path).arg(version_arg).output();
            match output {
                Ok(out) if out.status.success() || !out.stderr.is_empty() => {
                    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                    let combined = format!("{}{}", stdout, stderr);
                    let version = parse_version(&combined, cmd_name);

                    return BinaryStatus {
                        name: cmd_name.to_string(),
                        available: true,
                        version: Some(version),
                        path_or_sidecar: "app_data".to_string(),
                        absolute_path: Some(bin_path.to_string_lossy().to_string()),
                    };
                }
                _ => {}
            }
        }
    }

    // 3. Fallback to system PATH
    let output = std::process::Command::new(cmd_name).arg(version_arg).output();
    match output {
        Ok(out) if out.status.success() || !out.stderr.is_empty() => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let combined = format!("{}{}", stdout, stderr);
            let version = parse_version(&combined, cmd_name);

            if version != "Available" || out.status.success() {
                BinaryStatus {
                    name: cmd_name.to_string(),
                    available: true,
                    version: Some(version),
                    path_or_sidecar: "system_path".to_string(),
                    absolute_path: Some(cmd_name.to_string()),
                }
            } else {
                BinaryStatus {
                    name: cmd_name.to_string(),
                    available: false,
                    version: None,
                    path_or_sidecar: "not_found".to_string(),
                    absolute_path: None,
                }
            }
        }
        _ => BinaryStatus {
            name: cmd_name.to_string(),
            available: false,
            version: None,
            path_or_sidecar: "not_found".to_string(),
            absolute_path: None,
        },
    }
}

pub async fn get_binary_command<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    cmd_name: &str,
) -> Result<tauri_plugin_shell::process::Command, String> {
    // We use a simplified version of probe_sidecar logic without the version check
    // because we assume health check was already done or we just want to try running it.

    // 1. Try bundled sidecar
    if let Ok(sidecar) = app.shell().sidecar(cmd_name) {
        // We can't easily check if sidecar "exists" without running it in Tauri v2 shell plugin
        // but we can try to get its version as a quick check.
        // For performance, we could cache this, but for now let's just try to return it.
        return Ok(sidecar);
    }

    // 2. Try App Data Directory fallback
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        let bin_path = app_data_dir.join("bin").join(if cfg!(windows) {
            format!("{}.exe", cmd_name)
        } else {
            cmd_name.to_string()
        });

        if bin_path.exists() {
            return Ok(app.shell().command(bin_path.to_string_lossy().to_string()));
        }
    }

    // 3. Fallback to system PATH
    Ok(app.shell().command(cmd_name))
}

pub async fn check_sidecar_health<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> SidecarHealthReport {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_version_parsing() {
        assert_eq!(parse_version("ffmpeg version 6.0 Copyright...", "ffmpeg"), "6.0");
        assert_eq!(parse_version("pandoc 3.1.2", "pandoc"), "3.1.2");
        assert_eq!(parse_version("pdftotext version 23.05.0", "pdftotext"), "23.05.0");
        assert_eq!(parse_version("ImageMagick 7.1.1-15 Q16-HDRI...", "magick"), "7.1.1-15");
    }
}
