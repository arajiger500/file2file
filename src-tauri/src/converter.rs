use crate::formats::{get_category_for_extension, FileCategory};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri_plugin_shell::ShellExt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRequest {
    pub input_path: String,
    pub output_dir: Option<String>,
    pub target_format: String,
    pub crf: Option<u32>,
    pub resolution: Option<String>,
    pub hardware_accel: bool,
    pub selected_encoder: Option<String>,
    pub strip_metadata: bool,
    pub audio_bitrate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub job_id: String,
    pub input_path: String,
    pub output_path: String,
    pub success: bool,
    pub original_size_bytes: u64,
    pub converted_size_bytes: u64,
    pub elapsed_ms: u64,
    pub error: Option<String>,
}

pub async fn convert_single_file(
    app: tauri::AppHandle,
    req: ConversionRequest,
) -> Result<ConversionResult, String> {
    let input_path = Path::new(&req.input_path);
    if !input_path.exists() {
        return Err(format!("Input file does not exist: {}", req.input_path));
    }

    let input_ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let out_dir = match &req.output_dir {
        Some(d) => PathBuf::from(d),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let target_ext = req.target_format.to_lowercase();
    let output_filename = format!("{}_converted.{}", stem, target_ext);
    let output_path = out_dir.join(output_filename);

    let start_time = Instant::now();
    let original_size = std::fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);

    let category = get_category_for_extension(&input_ext);
    let target_category = get_category_for_extension(&target_ext);

    let execution_result = if input_ext == "pdf" {
        run_pdf_conversion(&app, input_path, &output_path, &target_ext).await
    } else if category == FileCategory::Image || category == FileCategory::Vector {
        run_image_magick_conversion(input_path, &output_path).await
    } else if target_category == FileCategory::Document {
        run_pandoc_conversion(&app, input_path, &output_path).await
    } else {
        run_ffmpeg_conversion(&app, &req, input_path, &output_path, &target_ext).await
    };

    let elapsed = start_time.elapsed().as_millis() as u64;

    match execution_result {
        Ok(_) => {
            let converted_size = std::fs::metadata(&output_path)
                .map(|m| m.len())
                .unwrap_or(0);
            Ok(ConversionResult {
                job_id: Uuid::new_v4().to_string(),
                input_path: req.input_path.clone(),
                output_path: output_path.to_string_lossy().to_string(),
                success: true,
                original_size_bytes: original_size,
                converted_size_bytes: converted_size,
                elapsed_ms: elapsed,
                error: None,
            })
        }
        Err(err) => Ok(ConversionResult {
            job_id: Uuid::new_v4().to_string(),
            input_path: req.input_path.clone(),
            output_path: output_path.to_string_lossy().to_string(),
            success: false,
            original_size_bytes: original_size,
            converted_size_bytes: 0,
            elapsed_ms: elapsed,
            error: Some(err),
        }),
    }
}

async fn run_pdf_conversion(
    app: &tauri::AppHandle,
    input: &Path,
    output: &Path,
    target_ext: &str,
) -> Result<(), String> {
    let out_dir = output.parent().unwrap_or_else(|| Path::new("."));

    let text_path = out_dir.join(format!(".file2file-{}.txt", Uuid::new_v4()));
    let extracted = std::process::Command::new("pdftotext")
        .args(["-layout", input.to_str().unwrap(), text_path.to_str().unwrap()])
        .output()
        .map_err(|_| "PDF conversion requires Poppler's pdftotext utility. Install poppler-utils and try again.".to_string())?;

    if !extracted.status.success() {
        return Err(String::from_utf8_lossy(&extracted.stderr)
            .trim()
            .to_string());
    }

    let result = if target_ext == "txt" {
        std::fs::rename(&text_path, output)
            .map_err(|e| format!("Could not save extracted text: {e}"))
    } else {
        let result = run_pandoc_conversion(app, &text_path, output).await;
        let _ = std::fs::remove_file(&text_path);
        result
    };

    result
}

async fn run_image_magick_conversion(input: &Path, output: &Path) -> Result<(), String> {
    let result = std::process::Command::new("magick")
        .args([input.to_str().unwrap(), output.to_str().unwrap()])
        .output()
        .map_err(|_| {
            "Image conversion requires ImageMagick (magick) to be installed.".to_string()
        })?;

    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).trim().to_string())
    }
}

async fn run_ffmpeg_conversion(
    app: &tauri::AppHandle,
    req: &ConversionRequest,
    input: &Path,
    output: &Path,
    target_ext: &str,
) -> Result<(), String> {
    if req.hardware_accel {
        let hw_res = execute_ffmpeg(app, req, input, output, target_ext, true).await;
        if hw_res.is_ok() {
            return Ok(());
        }
    }
    execute_ffmpeg(app, req, input, output, target_ext, false).await
}

async fn execute_ffmpeg(
    app: &tauri::AppHandle,
    req: &ConversionRequest,
    input: &Path,
    output: &Path,
    target_ext: &str,
    use_hw: bool,
) -> Result<(), String> {
    let mut args = vec![
        "-y".to_string(),
        "-i".to_string(),
        input.to_str().unwrap().to_string(),
    ];

    match target_ext {
        "mp4" | "mov" | "mkv" | "webm" | "avi" => {
            if use_hw {
                let enc = req.selected_encoder.as_deref().unwrap_or("h264_nvenc");
                args.push("-c:v".to_string());
                args.push(enc.to_string());
            } else {
                let codec = match target_ext {
                    "webm" => "libvpx-vp9",
                    "avi" => "mpeg4",
                    _ => "libx264",
                };
                args.push("-c:v".to_string());
                args.push(codec.to_string());
                if target_ext != "avi" {
                    args.push("-preset".to_string());
                    args.push("medium".to_string());
                }
            }
            if target_ext == "webm" {
                args.push("-c:a".to_string());
                args.push("libopus".to_string());
            } else if target_ext == "avi" {
                args.push("-c:a".to_string());
                args.push("libmp3lame".to_string());
            } else {
                args.push("-c:a".to_string());
                args.push("aac".to_string());
            }
        }
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "opus" => {
            args.push("-vn".to_string());
            match target_ext {
                "mp3" => {
                    args.push("-c:a".to_string());
                    args.push("libmp3lame".to_string());
                    args.push("-b:a".to_string());
                    args.push(req.audio_bitrate.as_deref().unwrap_or("320k").to_string());
                }
                "wav" => {
                    args.push("-c:a".to_string());
                    args.push("pcm_s16le".to_string());
                }
                "flac" => {
                    args.push("-c:a".to_string());
                    args.push("flac".to_string());
                }
                "opus" => {
                    args.push("-c:a".to_string());
                    args.push("libopus".to_string());
                }
                "ogg" => {
                    args.push("-c:a".to_string());
                    args.push("libvorbis".to_string());
                }
                "m4a" => {
                    args.push("-c:a".to_string());
                    args.push("aac".to_string());
                }
                _ => {
                    args.push("-c:a".to_string());
                    args.push("aac".to_string());
                }
            }
        }
        "webp" | "png" | "jpg" | "jpeg" | "avif" | "gif" | "bmp" | "tiff" => match target_ext {
            "webp" => {
                args.push("-c:v".to_string());
                args.push("libwebp".to_string());
            }
            "avif" => {
                args.push("-c:v".to_string());
                args.push("libaom-av1".to_string());
            }
            "gif" => {
                args.push("-vf".to_string());
                args.push("fps=15,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse".to_string());
            }
            _ => {}
        },
        _ => {}
    }

    if let Some(crf) = req.crf {
        args.push("-crf".to_string());
        args.push(crf.to_string());
    }

    if let Some(res) = &req.resolution {
        if res != "original" && !res.is_empty() {
            args.push("-vf".to_string());
            args.push(format!("scale={}", res.replace('x', ":")));
        }
    }

    if req.strip_metadata {
        args.push("-map_metadata".to_string());
        args.push("-1".to_string());
    }

    args.push(output.to_str().unwrap().to_string());

    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("Failed to create FFmpeg sidecar: {}", e))?
        .args(args)
        .output()
        .await
        .map_err(|e| format!("FFmpeg execution error: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn run_pandoc_conversion(
    app: &tauri::AppHandle,
    input: &Path,
    output: &Path,
) -> Result<(), String> {
    let output = app
        .shell()
        .sidecar("pandoc")
        .map_err(|e| format!("Failed to create Pandoc sidecar: {}", e))?
        .args(vec![
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("Pandoc execution error: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
