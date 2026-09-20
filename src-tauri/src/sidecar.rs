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
pub struct CategoryStatus {
    pub category: String,
    pub ready: bool,
    pub engine: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarHealthReport {
    pub binaries: Vec<BinaryStatus>,
    pub categories: Vec<CategoryStatus>,
    pub all_ready: bool,
    pub ffmpeg: BinaryStatus,
    pub ffprobe: BinaryStatus,
    pub pandoc: BinaryStatus,
    pub magick: BinaryStatus,
    pub imagemagick: BinaryStatus,
    pub pdftotext: BinaryStatus,
    pub pdftohtml: BinaryStatus,
}

fn parse_version(output: &str, cmd: &str) -> String {
    let first_line = output.lines().next().unwrap_or("Available");
    let version = match cmd {
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
        "pdftotext" | "pdftohtml" => {
            first_line
                .split("version ")
                .nth(1)
                .and_then(|s| s.split_whitespace().next())
                .unwrap_or(first_line)
                .to_string()
        }
        _ => first_line.to_string(),
    };
    version.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-').to_string()
}

pub async fn probe_sidecar<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    cmd_name: &str,
    version_arg: &str,
) -> BinaryStatus {
    println!("Probing binary: {}", cmd_name);

    // 1. Try bundled sidecar
    if let Ok(sidecar) = app.shell().sidecar(cmd_name) {
        println!("Checking bundled sidecar for {}", cmd_name);
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
            _ => {
                println!("Bundled sidecar check failed for {}", cmd_name);
            }
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
            println!("Checking app_data binary at: {:?}", bin_path);
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
                _ => {
                    println!("App_data binary execution failed for {}", cmd_name);
                }
            }
        }
    }

    // 3. Fallback to system PATH
    let actual_cmd = if cfg!(windows) {
        format!("{}.exe", cmd_name)
    } else {
        cmd_name.to_string()
    };

    println!("Checking system PATH for {}", actual_cmd);
    let output = std::process::Command::new(&actual_cmd).arg(version_arg).output();
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
                    absolute_path: Some(actual_cmd),
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
        _ => {
            // Last ditch effort for Windows without extension
            if cfg!(windows) {
                println!("Final fallback check for Windows without .exe: {}", cmd_name);
                let output = std::process::Command::new(cmd_name).arg(version_arg).output();
                if let Ok(out) = output {
                    if out.status.success() || !out.stderr.is_empty() {
                        let combined = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
                        return BinaryStatus {
                            name: cmd_name.to_string(),
                            available: true,
                            version: Some(parse_version(&combined, cmd_name)),
                            path_or_sidecar: "system_path".to_string(),
                            absolute_path: Some(cmd_name.to_string()),
                        };
                    }
                }
            }

            BinaryStatus {
                name: cmd_name.to_string(),
                available: false,
                version: None,
                path_or_sidecar: "not_found".to_string(),
                absolute_path: None,
            }
        },
    }
}

pub async fn get_binary_command<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    cmd_name: &str,
) -> Result<tauri_plugin_shell::process::Command, String> {
    let version_flag = match cmd_name {
        "pandoc" | "magick" => "--version",
        "pdftotext" | "pdftohtml" => "-v",
        _ => "-version",
    };

    // 1. Try bundled sidecar
    if let Ok(sidecar) = app.shell().sidecar(cmd_name) {
        let output = sidecar.arg(version_flag).output().await;
        if let Ok(out) = output {
             if out.status.success() || !out.stderr.is_empty() {
                return Ok(app.shell().sidecar(cmd_name).map_err(|e| e.to_string())?);
             }
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
            let output = std::process::Command::new(&bin_path).arg(version_flag).output();
            if let Ok(out) = output {
                if out.status.success() || !out.stderr.is_empty() {
                    return Ok(app.shell().command(bin_path.to_string_lossy().to_string()));
                }
            }
        }
    }

    // 3. Fallback to system PATH
    let actual_cmd = if cfg!(windows) {
        format!("{}.exe", cmd_name)
    } else {
        cmd_name.to_string()
    };

    if let Ok(out) = std::process::Command::new(&actual_cmd).arg(version_flag).output() {
        if out.status.success() || !out.stderr.is_empty() {
             return Ok(app.shell().command(actual_cmd));
        }
    }

    // Last ditch for Windows
    if cfg!(windows) {
        if let Ok(out) = std::process::Command::new(cmd_name).arg(version_flag).output() {
            if out.status.success() || !out.stderr.is_empty() {
                return Ok(app.shell().command(cmd_name.to_string()));
            }
        }
    }

    Err(format!(
        "Required dependency '{}' was not found. Please install it or place it in the application's bin folder.",
        cmd_name
    ))
}

pub struct CommandOutput {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub async fn spawn_and_track_simple<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &crate::AppState,
    job_id: &str,
    cmd_name: &str,
    args: Vec<String>,
) -> Result<CommandOutput, String> {
    let mut cmd = get_binary_command(app, cmd_name).await?;
    cmd = cmd.args(args);

    let (mut rx, child) = cmd.spawn().map_err(|e| format!("Failed to spawn {}: {}", cmd_name, e))?;
    state.active_jobs.lock().await.insert(job_id.to_string(), child);

    let timeout = std::time::Duration::from_secs(3600);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut success = false;

    let res = tokio::time::timeout(timeout, async {
        while let Some(event) = rx.recv().await {
            match event {
                tauri_plugin_shell::process::CommandEvent::Stdout(line) => stdout.extend(line),
                tauri_plugin_shell::process::CommandEvent::Stderr(line) => stderr.extend(line),
                tauri_plugin_shell::process::CommandEvent::Terminated(payload) => {
                    success = payload.code == Some(0);
                    break;
                }
                _ => {}
            }
        }
    }).await;

    state.active_jobs.lock().await.remove(job_id);

    if res.is_err() {
         return Err(format!("Process {} timed out", cmd_name));
    }

    Ok(CommandOutput { success, stdout, stderr })
}

pub async fn check_sidecar_health<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> SidecarHealthReport {
    let ffmpeg = probe_sidecar(app, "ffmpeg", "-version").await;
    let ffprobe = probe_sidecar(app, "ffprobe", "-version").await;
    let pandoc = probe_sidecar(app, "pandoc", "--version").await;
    let imagemagick = probe_sidecar(app, "magick", "--version").await;
    let pdftotext = probe_sidecar(app, "pdftotext", "-v").await;
    let pdftohtml = probe_sidecar(app, "pdftohtml", "-v").await;

    let binaries = vec![
        ffmpeg.clone(),
        ffprobe.clone(),
        pandoc.clone(),
        imagemagick.clone(),
        pdftotext.clone(),
        pdftohtml.clone(),
    ];

    let categories = vec![
        CategoryStatus {
            category: "Video".to_string(),
            ready: ffmpeg.available && ffprobe.available,
            engine: "FFmpeg".to_string(),
            message: if !ffmpeg.available { Some("FFmpeg missing".to_string()) } else { None },
        },
        CategoryStatus {
            category: "Audio".to_string(),
            ready: ffmpeg.available && ffprobe.available,
            engine: "FFmpeg".to_string(),
            message: if !ffmpeg.available { Some("FFmpeg missing".to_string()) } else { None },
        },
        CategoryStatus {
            category: "Images".to_string(),
            ready: imagemagick.available,
            engine: "ImageMagick".to_string(),
            message: if !imagemagick.available { Some("ImageMagick missing".to_string()) } else { None },
        },
        CategoryStatus {
            category: "Documents".to_string(),
            ready: pandoc.available && pdftotext.available && pdftohtml.available,
            engine: "Pandoc + Poppler".to_string(),
            message: if !pandoc.available { Some("Pandoc missing".to_string()) } else { None },
        },
        CategoryStatus {
            category: "Data".to_string(),
            ready: true,
            engine: "Rust-Native".to_string(),
            message: None,
        },
        CategoryStatus {
            category: "Archives".to_string(),
            ready: true,
            engine: "Rust-Native".to_string(),
            message: None,
        },
    ];

    let all_ready = binaries.iter().all(|b| b.available);

    SidecarHealthReport {
        binaries,
        categories,
        all_ready,
        ffmpeg,
        ffprobe,
        pandoc,
        magick: imagemagick.clone(),
        imagemagick,
        pdftotext,
        pdftohtml,
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
