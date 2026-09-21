use crate::process::{run, CommandOutput};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::Manager;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryStatus {
    pub name: String,
    pub available: bool,
    pub version: Option<String>,
    pub path_or_sidecar: String,
    pub absolute_path: Option<String>,
    pub error: Option<String>,
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
        "ffmpeg" | "ffprobe" => first_line
            .split("version ")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .unwrap_or(first_line)
            .to_string(),
        "magick" => first_line
            .split("ImageMagick ")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .unwrap_or(first_line)
            .to_string(),
        "pandoc" => first_line
            .split_whitespace()
            .nth(1)
            .unwrap_or(first_line)
            .to_string(),
        "pdftotext" | "pdftohtml" => first_line
            .split("version ")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .unwrap_or(first_line)
            .to_string(),
        _ => first_line.to_string(),
    };
    version
        .trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '-')
        .to_string()
}

const ENGINES: &[&str] = &[
    "ffmpeg",
    "ffprobe",
    "pandoc",
    "magick",
    "pdftotext",
    "pdftohtml",
];

fn executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    }
}

fn candidates<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    name: &str,
) -> Vec<(PathBuf, &'static str)> {
    let mut paths = Vec::new();
    let executable = executable_name(name);
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            paths.push((parent.join(&executable), "app_local"));
        }
    }
    if let Ok(data) = app.path().app_data_dir() {
        paths.push((data.join("bin").join(&executable), "app_data"));
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path).filter(|p| p.is_absolute()) {
            paths.push((dir.join(&executable), "system_path"));
        }
    }
    // ImageMagick 6 on Linux is named convert. Never use Windows' unrelated convert.exe.
    #[cfg(not(windows))]
    if name == "magick" {
        if let Some(path) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path).filter(|p| p.is_absolute()) {
                paths.push((dir.join("convert"), "system_path"));
            }
        }
    }
    paths
}

fn identity_matches(name: &str, output: &str) -> bool {
    match name {
        "ffmpeg" => output.starts_with("ffmpeg version "),
        "ffprobe" => output.starts_with("ffprobe version "),
        "pandoc" => output.starts_with("pandoc "),
        "magick" => output.contains("ImageMagick "),
        "pdftotext" => output.starts_with("pdftotext version "),
        "pdftohtml" => output.starts_with("pdftohtml version "),
        _ => false,
    }
}

pub async fn probe_sidecar<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    name: &str,
    flag: &str,
) -> BinaryStatus {
    let mut status = BinaryStatus {
        name: name.into(),
        available: false,
        version: None,
        path_or_sidecar: "not_found".into(),
        absolute_path: None,
        error: Some(format!(
            "Install {name}, then restart File2File or refresh engines."
        )),
    };
    if !ENGINES.contains(&name) {
        return status;
    }
    for (path, source) in candidates(app, name) {
        if !path.is_file() {
            continue;
        }
        let path = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => continue,
        };
        status.path_or_sidecar = source.into();
        status.absolute_path = Some(path.to_string_lossy().into());
        let mut command = Command::new(&path);
        command.arg(flag);
        match run(command, Duration::from_secs(5), 64 * 1024, None).await {
            Ok(out) => {
                let combined = format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                );
                if out.success && identity_matches(name, combined.trim()) {
                    status.available = true;
                    status.version = Some(parse_version(combined.trim(), name));
                    status.error = None;
                } else {
                    status.error = Some("Found but unusable: version probe failed or executable identity does not match.".into());
                }
            }
            Err(error) => status.error = Some(format!("Found but unusable: {error}")),
        }
        // Respect precedence. A broken app-local install must be visible, not silently bypassed.
        return status;
    }
    status
}

pub async fn get_binary_command<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    name: &str,
) -> Result<Command, String> {
    if !ENGINES.contains(&name) {
        return Err("Unknown engine".into());
    }
    let flag = match name {
        "pandoc" | "magick" => "--version",
        "pdftotext" | "pdftohtml" => "-v",
        _ => "-version",
    };
    let status = probe_sidecar(app, name, flag).await;
    if !status.available {
        return Err(format!("{name}: {}", status.error.unwrap_or_default()));
    }
    Ok(Command::new(Path::new(
        &status.absolute_path.ok_or("Engine path missing")?,
    )))
}

pub async fn spawn_and_track_simple<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &crate::AppState,
    job_id: &str,
    name: &str,
    args: Vec<String>,
) -> Result<CommandOutput, String> {
    state.check(job_id)?;
    let mut command = get_binary_command(app, name).await?;
    state.check(job_id)?;
    let image_policy = if name == "magick" {
        let directory = tempfile::Builder::new()
            .prefix("file2file-magick-policy-")
            .tempdir()
            .map_err(|e| format!("Cannot create ImageMagick policy directory: {e}"))?;
        std::fs::write(
            directory.path().join("policy.xml"),
            r#"<policymap>
  <policy domain="resource" name="memory" value="512MiB"/>
  <policy domain="resource" name="map" value="1GiB"/>
  <policy domain="resource" name="disk" value="2GiB"/>
  <policy domain="resource" name="width" value="32KP"/>
  <policy domain="resource" name="height" value="32KP"/>
  <policy domain="resource" name="area" value="128MP"/>
  <policy domain="resource" name="thread" value="4"/>
  <policy domain="resource" name="time" value="1800"/>
  <policy domain="delegate" rights="none" pattern="*"/>
  <policy domain="coder" rights="none" pattern="HTTP"/>
  <policy domain="coder" rights="none" pattern="HTTPS"/>
  <policy domain="coder" rights="none" pattern="URL"/>
  <policy domain="path" rights="none" pattern="@*"/>
</policymap>"#,
        )
        .map_err(|e| format!("Cannot write ImageMagick policy: {e}"))?;
        command.env("MAGICK_CONFIGURE_PATH", directory.path());
        command.env("MAGICK_TEMPORARY_PATH", directory.path());
        Some(directory)
    } else {
        None
    };
    command.args(args);
    let result = run(
        command,
        Duration::from_secs(3600),
        4 * 1024 * 1024,
        Some(state.subscribe(job_id)?),
    )
    .await;
    drop(image_policy);
    result
}

pub async fn check_sidecar_health<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> SidecarHealthReport {
    let (ffmpeg, ffprobe, pandoc, imagemagick, pdftotext, pdftohtml) = tokio::join!(
        probe_sidecar(app, "ffmpeg", "-version"),
        probe_sidecar(app, "ffprobe", "-version"),
        probe_sidecar(app, "pandoc", "--version"),
        probe_sidecar(app, "magick", "--version"),
        probe_sidecar(app, "pdftotext", "-v"),
        probe_sidecar(app, "pdftohtml", "-v"),
    );

    let binaries = vec![
        ffmpeg.clone(),
        ffprobe.clone(),
        pandoc.clone(),
        imagemagick.clone(),
        pdftotext.clone(),
        pdftohtml.clone(),
    ];

    let missing = |statuses: &[&BinaryStatus]| {
        let names: Vec<_> = statuses
            .iter()
            .filter(|status| !status.available)
            .map(|status| status.name.as_str())
            .collect();
        (!names.is_empty()).then(|| format!("Missing or unusable: {}", names.join(", ")))
    };

    let categories = vec![
        CategoryStatus {
            category: "Video".to_string(),
            ready: ffmpeg.available && ffprobe.available,
            engine: "FFmpeg".to_string(),
            message: missing(&[&ffmpeg, &ffprobe]),
        },
        CategoryStatus {
            category: "Audio".to_string(),
            ready: ffmpeg.available && ffprobe.available,
            engine: "FFmpeg".to_string(),
            message: missing(&[&ffmpeg, &ffprobe]),
        },
        CategoryStatus {
            category: "Images".to_string(),
            ready: imagemagick.available,
            engine: "ImageMagick".to_string(),
            message: missing(&[&imagemagick]),
        },
        CategoryStatus {
            category: "Documents".to_string(),
            ready: pandoc.available && pdftotext.available && pdftohtml.available,
            engine: "Pandoc + Poppler".to_string(),
            message: missing(&[&pandoc, &pdftotext, &pdftohtml]),
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
        assert_eq!(
            parse_version("ffmpeg version 6.0 Copyright...", "ffmpeg"),
            "6.0"
        );
        assert_eq!(parse_version("pandoc 3.1.2", "pandoc"), "3.1.2");
        assert_eq!(
            parse_version("pdftotext version 23.05.0", "pdftotext"),
            "23.05.0"
        );
        assert_eq!(
            parse_version("ImageMagick 7.1.1-15 Q16-HDRI...", "magick"),
            "7.1.1-15"
        );
    }
}
