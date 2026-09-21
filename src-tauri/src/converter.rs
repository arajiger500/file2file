use crate::errors::map_technical_error;
use crate::formats::{get_category_for_extension, FileCategory};
use crate::registry::Registry;
use crate::sidecar::{get_binary_command, spawn_and_track_simple};
use calamine::{Reader, Xlsx};
use rusqlite::Connection;
use rust_xlsxwriter::Workbook;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tauri::Manager;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CollisionPolicy {
    Overwrite,
    AutoRename,
    Skip,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionRequest {
    pub job_id: Option<String>,
    pub input_path: String,
    pub output_dir: Option<String>,
    pub target_format: String,
    pub crf: Option<u32>,
    pub resolution: Option<String>,
    pub hardware_accel: bool,
    pub selected_encoder: Option<String>,
    pub strip_metadata: bool,
    pub audio_bitrate: Option<String>,
    pub collision_policy: Option<CollisionPolicy>,
}

impl ConversionRequest {
    pub fn new(input_path: impl Into<String>, target_format: impl Into<String>) -> Self {
        Self {
            job_id: None,
            input_path: input_path.into(),
            output_dir: None,
            target_format: target_format.into(),
            crf: None,
            resolution: None,
            hardware_accel: false,
            selected_encoder: None,
            strip_metadata: false,
            audio_bitrate: None,
            collision_policy: None,
        }
    }
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
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProbeResult {
    pub has_video: bool,
    pub has_audio: bool,
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub format_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub error: Option<String>,
    pub file_info: Option<FileProbeResult>,
}

pub async fn probe_file_info<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    path: &str,
) -> Result<FileProbeResult, String> {
    let path = Path::new(path).canonicalize().map_err(|e| e.to_string())?;
    if !path.is_file() {
        return Err("Input must be a regular file".into());
    }
    let mut command = get_binary_command(app, "ffprobe").await?;
    command.args([
        "-v",
        "error",
        "-protocol_whitelist",
        "file,pipe",
        "-print_format",
        "json",
        "-show_format",
        "-show_streams",
        "-i",
    ]);
    command.arg(path);
    let output = crate::process::run(
        command,
        std::time::Duration::from_secs(15),
        4 * 1024 * 1024,
        None,
    )
    .await?;
    if !output.success {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let json: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let streams = json["streams"]
        .as_array()
        .ok_or("No streams found in file")?;
    let has_video = streams.iter().any(|s| s["codec_type"] == "video");
    let has_audio = streams.iter().any(|s| s["codec_type"] == "audio");

    let format = &json["format"];
    let duration = format["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let format_name = format["format_name"]
        .as_str()
        .unwrap_or("unknown")
        .to_string();

    let video_stream = streams.iter().find(|s| s["codec_type"] == "video");
    let width = video_stream.and_then(|s| s["width"].as_u64()).unwrap_or(0) as u32;
    let height = video_stream.and_then(|s| s["height"].as_u64()).unwrap_or(0) as u32;

    Ok(FileProbeResult {
        has_video,
        has_audio,
        duration,
        width,
        height,
        format_name,
    })
}

pub async fn validate_conversion<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    input_path_str: &str,
    target_format: &str,
) -> ValidationResult {
    let path = Path::new(input_path_str);
    if !path.exists() {
        return ValidationResult {
            is_valid: false,
            warnings: vec![],
            error: Some("Input file does not exist.".to_string()),
            file_info: None,
        };
    }

    let ext = if path.is_dir() {
        "folder".to_string()
    } else {
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
    };

    if !Registry::is_supported(&ext, target_format) {
        return ValidationResult {
            is_valid: false,
            warnings: vec![],
            error: Some(format!(
                "Conversion from .{} to .{} is not supported.",
                ext, target_format
            )),
            file_info: None,
        };
    }

    let category = get_category_for_extension(&ext);
    let target_cat = get_category_for_extension(target_format);

    let warnings = Vec::new();

    // Probe media files
    if category == FileCategory::Video || category == FileCategory::Audio {
        match probe_file_info(app, input_path_str).await {
            Ok(info) => {
                if target_cat == FileCategory::Audio && !info.has_audio {
                    return ValidationResult {
                        is_valid: false,
                        warnings: vec![],
                        error: Some(format!(
                            "This {} contains no audio track, so it cannot be converted to {}.",
                            category.to_string().to_lowercase(),
                            target_format.to_uppercase()
                        )),
                        file_info: Some(info),
                    };
                }

                if target_cat == FileCategory::Video && !info.has_video {
                    return ValidationResult {
                        is_valid: false,
                        warnings: vec![],
                        error: Some("The input contains no video stream.".to_string()),
                        file_info: Some(info),
                    };
                }

                return ValidationResult {
                    is_valid: true,
                    warnings,
                    error: None,
                    file_info: Some(info),
                };
            }
            Err(e) => {
                return ValidationResult {
                    is_valid: false,
                    warnings,
                    error: Some(format!("Media validation failed: {e}")),
                    file_info: None,
                };
            }
        }
    }

    ValidationResult {
        is_valid: true,
        warnings,
        error: None,
        file_info: None,
    }
}

pub async fn convert_single_file<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    req: ConversionRequest,
) -> Result<ConversionResult, String> {
    let input_path = Path::new(&req.input_path);
    if !input_path.exists() {
        return Err(format!("Input file does not exist: {}", req.input_path));
    }
    let input_path = input_path
        .canonicalize()
        .map_err(|e| format!("Input path resolution error: {}", e))?;

    let is_dir = input_path.is_dir();
    if !input_path.is_file() && !is_dir {
        return Err("Input must be a regular file or directory".to_string());
    }

    let input_ext = if is_dir {
        "folder".to_string()
    } else {
        input_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase()
    };

    let target_ext = req.target_format.to_lowercase();

    // 1. Validation against registry and parameters
    validate_request_parameters(&input_ext, &target_ext, &req)?;

    let job_id = req
        .job_id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    if app.try_state::<crate::AppState>().is_none() {
        app.manage(crate::AppState::default());
    }
    let state_holder = app.state::<crate::AppState>();
    let state = state_holder.inner();
    let _job = state.register(&job_id)?;
    let mut cancellation = state.subscribe(&job_id)?;
    let _permit = tokio::select! {
        biased;
        _ = crate::process::cancelled(&mut cancellation) => return Err("Conversion cancelled".into()),
        permit = state.conversion_semaphore.acquire() => permit.map_err(|e| e.to_string())?,
    };
    state.check(&job_id)?;

    let stem = if is_dir {
        input_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("archive")
            .to_string()
    } else {
        input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string()
    };

    let safe_stem = stem
        .chars()
        .take(60)
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let safe_stem = if safe_stem.is_empty() {
        "output".to_string()
    } else {
        safe_stem
    };

    let out_dir = match &req.output_dir {
        Some(d) => {
            let p = PathBuf::from(d);
            if !p.exists() {
                std::fs::create_dir_all(&p)
                    .map_err(|e| format!("Failed to create output directory: {}", e))?;
            }
            p.canonicalize()
                .map_err(|e| format!("Output path error: {}", e))?
        }
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    if !out_dir.is_dir() {
        return Err("Output root is not a directory".into());
    }
    if is_dir && out_dir.starts_with(&input_path) {
        return Err("Archive output must be outside the input folder".into());
    }
    let policy = req
        .collision_policy
        .clone()
        .unwrap_or(CollisionPolicy::AutoRename);
    let temp_dir = tempfile::Builder::new()
        .prefix(".file2file-")
        .tempdir_in(&out_dir)
        .map_err(|e| format!("Failed to create output workspace: {e}"))?;
    let temp_output = if target_ext == "folder" {
        temp_dir.path().join("extracted_out")
    } else {
        temp_dir.path().join(format!("work.{}", target_ext))
    };

    let start_time = Instant::now();
    let original_size = if is_dir {
        0
    } else {
        std::fs::metadata(&input_path).map(|m| m.len()).unwrap_or(0)
    };

    let category = get_category_for_extension(&input_ext);
    let target_category = get_category_for_extension(&target_ext);

    let execution_result = if input_ext == "pdf" {
        run_pdf_conversion(&app, state, &job_id, &input_path, &temp_output, &target_ext)
            .await
            .map(|_| vec![])
    } else if category == FileCategory::Data {
        let (input, output, from, to) = (
            input_path.clone(),
            temp_output.clone(),
            input_ext.clone(),
            target_ext.clone(),
        );
        tokio::task::spawn_blocking(move || run_data_conversion_sync(&input, &output, &from, &to))
            .await
            .map_err(|e| e.to_string())?
            .map(|_| vec![])
    } else if category == FileCategory::Archive || input_ext == "folder" || input_ext == "directory"
    {
        let (input, output, from, to) = (
            input_path.clone(),
            temp_output.clone(),
            input_ext.clone(),
            target_ext.clone(),
        );
        let cancellation = state.subscribe(&job_id)?;
        tokio::task::spawn_blocking(move || {
            crate::archive::convert(&input, &output, &from, &to, Some(cancellation))
        })
        .await
        .map_err(|e| e.to_string())?
        .map(|_| vec![])
    } else if category == FileCategory::Image || category == FileCategory::Vector {
        if target_ext == "pdf" || get_category_for_extension(&target_ext) == FileCategory::Image {
            run_image_magick_conversion(&app, state, &job_id, &input_path, &temp_output)
                .await
                .map(|_| vec![])
        } else {
            run_pandoc_conversion(
                &app,
                state,
                &job_id,
                &input_path,
                &temp_output,
                temp_dir.path(),
            )
            .await
            .map(|_| vec![])
        }
    } else if target_category == FileCategory::Document {
        run_pandoc_conversion(
            &app,
            state,
            &job_id,
            &input_path,
            &temp_output,
            temp_dir.path(),
        )
        .await
        .map(|_| vec![])
    } else {
        run_ffmpeg_conversion(
            &app,
            state,
            &job_id,
            &req,
            &input_path,
            &temp_output,
            &target_ext,
        )
        .await
    };

    state.check(&job_id)?;
    let elapsed = start_time.elapsed().as_millis() as u64;

    match execution_result {
        Ok(mut warnings) => {
            let (output_path, skipped) = state.publish(&job_id, || {
                crate::output::publish(&temp_output, &out_dir, &safe_stem, &target_ext, &policy)
            })?;
            if skipped {
                warnings
                    .push("Existing output preserved; conversion skipped at publication.".into());
            }
            let converted_size = fs::metadata(&output_path)
                .map(|m| if m.is_file() { m.len() } else { 0 })
                .unwrap_or(0);
            Ok(ConversionResult {
                job_id,
                input_path: req.input_path.clone(),
                output_path: output_path.to_string_lossy().to_string(),
                success: true,
                original_size_bytes: original_size,
                converted_size_bytes: converted_size,
                elapsed_ms: elapsed,
                error: None,
                warnings,
            })
        }
        Err(err) => {
            let friendly_err = map_technical_error(&err, &req.input_path, &req.target_format);
            Ok(ConversionResult {
                job_id,
                input_path: req.input_path.clone(),
                output_path: String::new(),
                success: false,
                original_size_bytes: original_size,
                converted_size_bytes: 0,
                elapsed_ms: elapsed,
                error: Some(friendly_err),
                warnings: vec![],
            })
        }
    }
}

fn validate_request_parameters(
    input_ext: &str,
    target_ext: &str,
    req: &ConversionRequest,
) -> Result<(), String> {
    if !Registry::is_supported(input_ext, target_ext) {
        return Err(format!(
            "Unsupported conversion path: .{} to .{}",
            input_ext, target_ext
        ));
    }

    if let Some(encoder) = req.selected_encoder.as_deref() {
        if ![
            "auto",
            "libx264",
            "libx265",
            "h264_nvenc",
            "hevc_nvenc",
            "h264_qsv",
            "h264_vaapi",
            "h264_videotoolbox",
        ]
        .contains(&encoder)
        {
            return Err("Unsupported video encoder".into());
        }
    }
    if let Some(crf) = req.crf {
        if crf > 51 {
            return Err("CRF value out of range (0-51)".to_string());
        }
    }
    if let Some(res) = &req.resolution {
        if res != "original" && !res.is_empty() {
            let re = regex::Regex::new(r"^\d+x\d+$").map_err(|_| "Regex err")?;
            if !re.is_match(res)
                || res
                    .split('x')
                    .any(|n| n.parse::<u32>().map_or(true, |v| v == 0 || v > 8192))
            {
                return Err("Invalid resolution format. Expected 'WxH' or 'original'.".to_string());
            }
        }
    }
    if let Some(br) = &req.audio_bitrate {
        let re = regex::Regex::new(r"^\d+k$").map_err(|_| "Regex err")?;
        if !re.is_match(br)
            || br
                .trim_end_matches('k')
                .parse::<u32>()
                .map_or(true, |v| v == 0 || v > 1536)
        {
            return Err("Invalid audio bitrate format. Expected 'Nk' (e.g. 192k).".to_string());
        }
    }
    Ok(())
}

async fn run_pdf_conversion<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &crate::AppState,
    job_id: &str,
    input: &Path,
    output: &Path,
    target_ext: &str,
) -> Result<(), String> {
    let out_dir = output.parent().unwrap_or_else(|| Path::new("."));
    let input_str = input.to_str().ok_or("Input path contains invalid UTF-8")?;

    if target_ext == "txt" {
        let text_path = out_dir.join(format!(".file2file-{}.txt", Uuid::new_v4()));
        let text_path_str = text_path
            .to_str()
            .ok_or("Temporary path contains invalid UTF-8")?;

        let extracted = spawn_and_track_simple(
            app,
            state,
            job_id,
            "pdftotext",
            vec![
                "-layout".to_string(),
                input_str.to_string(),
                text_path_str.to_string(),
            ],
        )
        .await?;

        if !extracted.success {
            let _ = std::fs::remove_file(&text_path);
            return Err(String::from_utf8_lossy(&extracted.stderr)
                .trim()
                .to_string());
        }

        let res = std::fs::copy(&text_path, output)
            .map(|_| ())
            .map_err(|e| format!("Could not save extracted text: {e}"));
        let _ = std::fs::remove_file(&text_path);
        res
    } else {
        // High-fidelity extraction using pdftohtml + pandoc
        let temp_prefix = out_dir.join(format!(".file2file-{}", Uuid::new_v4()));
        let temp_prefix_str = temp_prefix
            .to_str()
            .ok_or("Temporary prefix contains invalid UTF-8")?;

        let output_gen = spawn_and_track_simple(
            app,
            state,
            job_id,
            "pdftohtml",
            vec![
                "-noframes".to_string(),
                input_str.to_string(),
                temp_prefix_str.to_string(),
            ],
        )
        .await?;

        if !output_gen.success {
            return Err(String::from_utf8_lossy(&output_gen.stderr)
                .trim()
                .to_string());
        }

        let temp_html_name = temp_prefix
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Temporary file name error")?;
        let temp_html = out_dir.join(format!("{}.html", temp_html_name));

        if !temp_html.exists() {
            return Err("pdftohtml completed but generated HTML was not found.".to_string());
        }

        let pandoc_res =
            run_pandoc_conversion(app, state, job_id, &temp_html, output, out_dir).await;
        let _ = std::fs::remove_file(&temp_html);
        pandoc_res
    }
}

async fn run_image_magick_conversion<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &crate::AppState,
    job_id: &str,
    input: &Path,
    output: &Path,
) -> Result<(), String> {
    let input_str = input.to_str().ok_or("Input path contains invalid UTF-8")?;
    let output_str = output
        .to_str()
        .ok_or("Output path contains invalid UTF-8")?;

    let output_ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let input_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut args = Vec::new();

    // Add density for vector formats to ensure high quality rasterization
    if matches!(input_ext.as_str(), "svg" | "eps" | "ai" | "pdf") {
        args.push("-density".to_string());
        args.push("300".to_string());
    }

    args.push(input_str.to_string());

    if input_ext == "gif" || input_ext == "webp" {
        args.push("-coalesce".to_string());
    }

    if output_ext == "jpg" || output_ext == "jpeg" {
        // Flatten alpha against white background for clean JPG output
        args.push("-background".to_string());
        args.push("white".to_string());
        args.push("-flatten".to_string());
    } else if output_ext == "ico" {
        // Multi-resolution icon generation for Windows ICO
        args.push("-define".to_string());
        args.push("icon:auto-resize=256,128,64,48,32,16".to_string());
    }

    args.push(output_str.to_string());

    let result = spawn_and_track_simple(app, state, job_id, "magick", args).await?;

    if result.success {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).trim().to_string())
    }
}

async fn run_ffmpeg_conversion<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &crate::AppState,
    job_id: &str,
    req: &ConversionRequest,
    input: &Path,
    output: &Path,
    target_ext: &str,
) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    let paths = FfmpegPaths {
        input,
        output,
        target_ext,
    };

    if req.hardware_accel && target_ext != "webm" {
        let hw_res = execute_ffmpeg(app, state, job_id, req, &paths, true).await;
        if hw_res.is_ok() {
            return Ok(warnings);
        }
        state.check(job_id)?;
        warnings.push("Hardware acceleration failed; fell back to software encoding.".to_string());
    }

    execute_ffmpeg(app, state, job_id, req, &paths, false)
        .await
        .map(|_| warnings)
}

struct FfmpegPaths<'a> {
    input: &'a Path,
    output: &'a Path,
    target_ext: &'a str,
}

async fn execute_ffmpeg<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &crate::AppState,
    job_id: &str,
    req: &ConversionRequest,
    paths: &FfmpegPaths<'_>,
    use_hw: bool,
) -> Result<(), String> {
    let input = paths.input;
    let output = paths.output;
    let target_ext = paths.target_ext;
    let input_str = input.to_str().ok_or("Input path contains invalid UTF-8")?;
    let output_str = output
        .to_str()
        .ok_or("Output path contains invalid UTF-8")?;

    let is_audio_target = matches!(
        target_ext,
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "opus"
    );
    let is_video_target = matches!(target_ext, "mp4" | "mov" | "mkv" | "webm" | "avi");
    let is_gif_target = target_ext == "gif";

    let mut args = vec![
        "-nostdin".into(),
        "-v".into(),
        "error".into(),
        "-y".into(),
        "-protocol_whitelist".into(),
        "file,pipe".into(),
        "-i".into(),
        input_str.into(),
    ];

    if is_audio_target {
        // Strip video stream
        args.push("-vn".to_string());
        // Map audio stream optionally
        args.push("-map".to_string());
        args.push("0:a:0".to_string());

        let bitrate = req.audio_bitrate.as_deref().unwrap_or("256k");

        match target_ext {
            "mp3" => {
                args.push("-c:a".to_string());
                args.push("libmp3lame".to_string());
                args.push("-b:a".to_string());
                args.push(bitrate.to_string());
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
                args.push("-b:a".to_string());
                args.push(bitrate.to_string());
            }
            "ogg" => {
                args.push("-c:a".to_string());
                args.push("libvorbis".to_string());
                args.push("-b:a".to_string());
                args.push(bitrate.to_string());
            }
            "m4a" | "aac" => {
                args.push("-c:a".to_string());
                args.push("aac".to_string());
                args.push("-b:a".to_string());
                args.push(bitrate.to_string());
            }
            _ => {
                args.push("-c:a".to_string());
                args.push("aac".to_string());
            }
        }
    } else if is_video_target {
        // Map video stream and optional audio stream
        args.push("-map".to_string());
        args.push("0:v:0".to_string());
        args.push("-map".to_string());
        args.push("0:a?".to_string());
        args.push("-map".to_string());
        args.push("0:s?".to_string());

        // Video codec
        if use_hw {
            let enc = req
                .selected_encoder
                .as_deref()
                .filter(|s| *s != "auto")
                .unwrap_or("libx264");

            // Basic mismatch prevention: don't use h264 for webm
            let actual_enc = if target_ext == "webm" && enc.contains("264") {
                "vp9_nvenc" // or we just let it fail and fallback to sw
            } else {
                enc
            };

            args.push("-c:v".to_string());
            args.push(actual_enc.to_string());
            if target_ext == "mp4" || target_ext == "mov" {
                args.push("-pix_fmt".to_string());
                args.push("yuv420p".to_string());
            }

            if let Some(crf) = req.crf {
                if actual_enc.contains("nvenc") {
                    args.push("-cq".to_string());
                    args.push(crf.to_string());
                } else if actual_enc.contains("videotoolbox") {
                    args.push("-q:v".to_string());
                    args.push((crf * 2).to_string()); // just a rough mapping
                } else {
                    args.push("-global_quality".to_string());
                    args.push(crf.to_string());
                }
            }
        } else {
            let codec = match target_ext {
                "webm" => "libvpx-vp9",
                "avi" => "mpeg4",
                _ => "libx264",
            };
            args.push("-c:v".to_string());
            args.push(codec.to_string());
            if codec == "libx264" {
                args.push("-pix_fmt".to_string());
                args.push("yuv420p".to_string());
                args.push("-preset".to_string());
                args.push("medium".to_string());
            }
            if let Some(crf) = req.crf {
                args.push("-crf".to_string());
                args.push(crf.to_string());
            }
        }

        // Audio codec
        match target_ext {
            "webm" => {
                args.push("-c:a".to_string());
                args.push("libopus".to_string());
            }
            "avi" => {
                args.push("-c:a".to_string());
                args.push("libmp3lame".to_string());
            }
            _ => {
                args.push("-c:a".to_string());
                args.push("aac".to_string());
            }
        }

        // Subtitles
        match target_ext {
            "mkv" => {
                args.push("-c:s".to_string());
                args.push("copy".to_string());
            }
            "mp4" => {
                args.push("-c:s".to_string());
                args.push("mov_text".to_string());
            }
            "webm" => {
                args.push("-c:s".to_string());
                args.push("webvtt".to_string());
            }
            _ => {
                args.push("-sn".to_string());
            }
        }

        // Video resolution scale
        if let Some(res) = &req.resolution {
            if res != "original" && !res.is_empty() {
                args.push("-vf".to_string());
                args.push(format!("scale={}", res.replace('x', ":")));
            }
        }
    } else if target_ext == "srt" {
        args.push("-vn".to_string());
        args.push("-an".to_string());
        args.push("-map".to_string());
        args.push("0:s:0".to_string());
        args.push("-c:s".to_string());
        args.push("srt".to_string());
    } else if is_gif_target {
        // Strip audio for GIF
        args.push("-an".to_string());
        let scale_filter = if let Some(res) = &req.resolution {
            if res != "original" && !res.is_empty() {
                format!("scale={}:flags=lanczos", res.replace('x', ":"))
            } else {
                "scale=iw:ih:flags=lanczos".to_string()
            }
        } else {
            "scale=iw:ih:flags=lanczos".to_string()
        };
        // High quality two-pass palette GIF filter
        args.push("-vf".to_string());
        args.push(format!(
            "fps=12,{},split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse",
            scale_filter
        ));
    }

    // Metadata handling
    if req.strip_metadata {
        args.push("-map_metadata".to_string());
        args.push("-1".to_string());
    } else {
        args.push("-map_metadata".to_string());
        args.push("0".to_string());
    }

    args.push(output_str.to_string());

    let result = spawn_and_track_simple(app, state, job_id, "ffmpeg", args).await?;

    if result.success {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).to_string())
    }
}

async fn run_pandoc_conversion<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &crate::AppState,
    job_id: &str,
    input: &Path,
    output: &Path,
    work_dir: &Path,
) -> Result<(), String> {
    let input_str = input.to_str().ok_or("Input path contains invalid UTF-8")?;
    let output_str = output
        .to_str()
        .ok_or("Output path contains invalid UTF-8")?;

    let input_ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let output_ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut args = vec![
        input_str.to_string(),
        "-o".to_string(),
        output_str.to_string(),
    ];

    args.push("--sandbox".into());
    if output_ext == "txt" {
        args.extend(["-t".into(), "plain".into()]);
    }

    // Infer accurate reader format
    match input_ext.as_str() {
        "md" | "markdown" => {
            args.push("-f".to_string());
            args.push("markdown+tex_math_dollars".to_string());
        }
        "html" | "htm" => {
            args.push("-f".to_string());
            args.push("html".to_string());
        }
        "docx" => {
            args.push("-f".to_string());
            args.push("docx".to_string());
        }
        "odt" => {
            args.push("-f".to_string());
            args.push("odt".to_string());
        }
        "epub" => {
            args.push("-f".to_string());
            args.push("epub".to_string());
        }
        "rtf" => {
            args.push("-f".to_string());
            args.push("rtf".to_string());
        }
        "txt" => {
            args.push("-f".to_string());
            args.push("markdown".to_string());
        }
        _ => {}
    }

    if output_ext == "html" || output_ext == "htm" {
        args.push("--embed-resources".to_string());
        args.push("--standalone".to_string());
    } else if matches!(output_ext.as_str(), "docx" | "odt" | "epub") {
        let media_dir = work_dir.join("media");
        args.push(format!("--extract-media={}", media_dir.to_string_lossy()));
    }

    let result = spawn_and_track_simple(app, state, job_id, "pandoc", args).await?;

    if result.success {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).to_string())
    }
}

fn run_data_conversion_sync(
    input: &Path,
    output: &Path,
    input_ext: &str,
    target_ext: &str,
) -> Result<(), String> {
    if fs::metadata(input).map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
        return Err("Structured input exceeds 32 MiB safety limit".into());
    }
    let input_file = File::open(input).map_err(|e| format!("Failed to open input file: {}", e))?;
    let reader = BufReader::new(input_file);

    let value: Value = match input_ext {
        "json" => {
            serde_json::from_reader(reader).map_err(|e| format!("JSON parse error: {}", e))?
        }
        "csv" => {
            let mut csv_reader = csv::Reader::from_reader(reader);
            let headers = csv_reader
                .headers()
                .map_err(|e| format!("CSV headers error: {}", e))?
                .clone();
            validate_headers(headers.iter())?;
            let mut items = Vec::new();
            for result in csv_reader.records() {
                let record = result.map_err(|e| format!("CSV record error: {}", e))?;
                let mut map = serde_json::Map::new();
                for (header, cell) in headers.iter().zip(record.iter()) {
                    let parsed_val = parse_inferred_scalar(cell);
                    map.insert(header.to_string(), parsed_val);
                }
                items.push(Value::Object(map));
            }
            Value::Array(items)
        }
        "yaml" => {
            serde_yaml::from_reader(reader).map_err(|e| format!("YAML parse error: {}", e))?
        }
        "toml" => {
            let content = std::fs::read_to_string(input)
                .map_err(|e| format!("Failed to read input file: {}", e))?;
            toml::from_str(&content).map_err(|e| format!("TOML parse error: {}", e))?
        }
        "xml" => {
            let content = std::fs::read_to_string(input)
                .map_err(|e| format!("Failed to read input file: {}", e))?;
            quick_xml::de::from_str(&content).map_err(|e| format!("XML parse error: {}", e))?
        }
        "xlsx" => {
            check_xlsx_size(input)?;
            let mut excel: Xlsx<_> =
                calamine::open_workbook(input).map_err(|e| format!("Excel open error: {}", e))?;
            let sheet_name = excel
                .sheet_names()
                .first()
                .ok_or("No sheets in excel")?
                .clone();
            let range = excel
                .worksheet_range(&sheet_name)
                .map_err(|e| format!("Range error: {}", e))?;

            let mut items = Vec::new();
            let mut rows = range.rows();
            let headers: Vec<String> = rows
                .next()
                .ok_or("Empty excel sheet")?
                .iter()
                .map(|c| c.to_string())
                .collect();
            validate_headers(headers.iter().map(String::as_str))?;

            for row in rows {
                let mut map = serde_json::Map::new();
                for (header, cell) in headers.iter().zip(row.iter()) {
                    let val = match cell {
                        calamine::Data::Empty => Value::Null,
                        calamine::Data::String(s) => Value::String(s.clone()),
                        calamine::Data::Float(f) => serde_json::Number::from_f64(*f)
                            .map(Value::Number)
                            .unwrap_or(Value::Null),
                        calamine::Data::Int(i) => Value::Number((*i).into()),
                        calamine::Data::Bool(b) => Value::Bool(*b),
                        _ => Value::String(cell.to_string()),
                    };
                    map.insert(header.clone(), val);
                }
                items.push(Value::Object(map));
            }
            Value::Array(items)
        }
        _ => Value::Null,
    };

    if matches!(target_ext, "csv" | "xlsx") {
        let rows = value
            .as_array()
            .ok_or("Tabular output requires an array of objects")?;
        if rows.is_empty() || rows.iter().any(|row| !row.is_object()) {
            return Err(
                "Tabular output requires a nonempty array of objects; scalar rows would lose data"
                    .into(),
            );
        }
    }
    validate_value(&value, 0)?;
    match (input_ext, target_ext) {
        (i, "xlsx") if i != "xlsx" && value != Value::Null => {
            let mut workbook = Workbook::new();
            let worksheet = workbook.add_worksheet();
            if let Some(arr) = value.as_array() {
                if !arr.is_empty() {
                    let headers: Vec<String> = arr
                        .iter()
                        .filter_map(Value::as_object)
                        .flat_map(|obj| obj.keys().cloned())
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    if headers.len() > 16_384 || arr.len() > 1_048_575 {
                        return Err("Data exceeds XLSX row or column limits".into());
                    }
                    for (col, header) in headers.iter().enumerate() {
                        worksheet
                            .write_string(0, col as u16, header)
                            .map_err(|e| format!("Excel header write error: {}", e))?;
                    }
                    for (row_idx, item) in arr.iter().enumerate() {
                        if let Some(obj) = item.as_object() {
                            for (col_idx, header) in headers.iter().enumerate() {
                                if let Some(val) = obj.get(header) {
                                    match val {
                                        Value::Number(n) => {
                                            if n.as_i64().is_some_and(|v| {
                                                v.unsigned_abs() > 9_007_199_254_740_991
                                            }) || n
                                                .as_u64()
                                                .is_some_and(|v| v > 9_007_199_254_740_991)
                                            {
                                                worksheet
                                                    .write_string(
                                                        row_idx as u32 + 1,
                                                        col_idx as u16,
                                                        n.to_string(),
                                                    )
                                                    .map_err(|e| e.to_string())?;
                                            } else if let Some(f) = n.as_f64() {
                                                worksheet
                                                    .write(row_idx as u32 + 1, col_idx as u16, f)
                                                    .map_err(|e| {
                                                        format!("Excel write error: {}", e)
                                                    })?;
                                            }
                                        }
                                        Value::String(s) => {
                                            worksheet
                                                .write_string(row_idx as u32 + 1, col_idx as u16, s)
                                                .map_err(|e| format!("Excel write error: {}", e))?;
                                        }
                                        Value::Bool(b) => {
                                            worksheet
                                                .write(row_idx as u32 + 1, col_idx as u16, *b)
                                                .map_err(|e| format!("Excel write error: {}", e))?;
                                        }
                                        _ => {
                                            worksheet
                                                .write(
                                                    row_idx as u32 + 1,
                                                    col_idx as u16,
                                                    val.to_string(),
                                                )
                                                .map_err(|e| format!("Excel write error: {}", e))?;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            workbook
                .save(output)
                .map_err(|e| format!("Failed to save XLSX: {}", e))?;
            return Ok(());
        }
        (i, "json") if i != "json" && value != Value::Null => {
            let json = serde_json::to_string_pretty(&value)
                .map_err(|e| format!("JSON serialization error: {}", e))?;
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
            return Ok(());
        }
        (i, "xml") if i != "xml" && value != Value::Null => {
            let mut buf = String::new();
            buf.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root>\n");
            validate_xml(&value)?;
            val_to_xml(&value, &mut buf, 1);
            buf.push_str("</root>\n");
            std::fs::write(output, buf).map_err(|e| format!("Failed to write output: {}", e))?;
            return Ok(());
        }
        (i, "yaml") if i != "yaml" && value != Value::Null => {
            let yaml = serde_yaml::to_string(&value)
                .map_err(|e| format!("YAML serialization error: {}", e))?;
            std::fs::write(output, yaml).map_err(|e| format!("Failed to write output: {}", e))?;
            return Ok(());
        }
        (i, "toml") if i != "toml" && value != Value::Null => {
            // TOML root must be a table (Map)
            let toml_val = if value.is_array() {
                let mut map = serde_json::Map::new();
                map.insert("items".to_string(), value);
                Value::Object(map)
            } else {
                value
            };
            let toml_string = toml::to_string(&toml_val)
                .map_err(|e| format!("TOML serialization error: {}", e))?;
            std::fs::write(output, toml_string)
                .map_err(|e| format!("Failed to write output: {}", e))?;
            return Ok(());
        }
        (i, "csv") if i != "csv" && value != Value::Null => {
            if let Some(arr) = value.as_array() {
                if arr.is_empty() {
                    std::fs::write(output, "")
                        .map_err(|e| format!("Failed to write CSV: {}", e))?;
                    return Ok(());
                }
                let mut headers = std::collections::BTreeSet::new();
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        for key in obj.keys() {
                            headers.insert(key.to_string());
                        }
                    }
                }
                let header_list: Vec<String> = headers.into_iter().collect();
                let mut wtr = csv::Writer::from_path(output)
                    .map_err(|e| format!("Failed to create CSV writer: {}", e))?;
                wtr.write_record(
                    header_list
                        .iter()
                        .map(|cell| safe_csv_cell(cell).into_owned()),
                )
                .map_err(|e| format!("CSV header write error: {}", e))?;
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        let row: Vec<String> = header_list
                            .iter()
                            .map(|h| {
                                obj.get(h)
                                    .map(|v| match v {
                                        Value::String(s) => s.clone(),
                                        Value::Null => String::new(),
                                        _ => v.to_string(),
                                    })
                                    .unwrap_or_default()
                            })
                            .collect();
                        wtr.write_record(row.iter().map(|cell| safe_csv_cell(cell).into_owned()))
                            .map_err(|e| format!("CSV row write error: {}", e))?;
                    }
                }
                wtr.flush()
                    .map_err(|e| format!("Failed to flush CSV: {}", e))?;
            } else {
                return Err("Input must be an array of objects to convert to CSV".to_string());
            }
            return Ok(());
        }
        ("csv", "sql") => {
            let file = File::open(input).map_err(|e| format!("Failed to open CSV: {}", e))?;
            let mut csv_reader = csv::Reader::from_reader(file);
            let headers = csv_reader
                .headers()
                .map_err(|e| format!("CSV headers error: {}", e))?
                .clone();
            let raw_stem = input
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("imported_table");
            let table_name = raw_stem
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect::<String>();
            let safe_table = if table_name.is_empty() {
                "imported_table".to_string()
            } else {
                table_name
            };

            let mut sql = format!(
                "CREATE TABLE IF NOT EXISTS \"{}\" (\n",
                safe_table.replace('"', "\"\"")
            );
            let mut sql_columns = std::collections::HashSet::new();
            for (i, h) in headers.iter().enumerate() {
                let col = h
                    .chars()
                    .map(|c| {
                        if c.is_alphanumeric() || c == '_' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect::<String>();
                let safe_col = if col.is_empty() {
                    format!("col_{}", i)
                } else {
                    col
                };
                if !sql_columns.insert(safe_col.to_lowercase()) {
                    return Err(
                        "CSV column names collide after SQL identifier normalization".into(),
                    );
                }
                sql.push_str(&format!(
                    "  \"{}\" TEXT{}",
                    safe_col.replace('"', "\"\""),
                    if i < headers.len() - 1 { "," } else { "" }
                ));
                sql.push('\n');
            }
            sql.push_str(");\n\n");

            for result in csv_reader.records() {
                let record = result.map_err(|e| format!("CSV record error: {}", e))?;
                sql.push_str(&format!(
                    "INSERT INTO \"{}\" VALUES (",
                    safe_table.replace('"', "\"\"")
                ));
                for (i, val) in record.iter().enumerate() {
                    if val.is_empty() {
                        sql.push_str("NULL");
                    } else {
                        sql.push_str(&format!("'{}'", val.replace('\'', "''")));
                    }
                    if i < record.len() - 1 {
                        sql.push_str(", ");
                    }
                }
                sql.push_str(");\n");
            }
            std::fs::write(output, sql)
                .map_err(|e| format!("Failed to write SQL output: {}", e))?;
            return Ok(());
        }
        ("log", "json") => {
            let mut content = String::new();
            File::open(input)
                .map_err(|e| format!("Open log error: {}", e))?
                .read_to_string(&mut content)
                .map_err(|e| format!("Read log error: {}", e))?;

            let log_re =
                regex::Regex::new(r"(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})\s+(\w+)\s+(.*)")
                    .map_err(|e| format!("Regex compile error: {}", e))?;

            let mut items = Vec::new();
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                let mut map = serde_json::Map::new();
                map.insert("raw".to_string(), Value::String(line.to_string()));
                if let Some(caps) = log_re.captures(line) {
                    if let Some(ts) = caps.get(1) {
                        map.insert(
                            "timestamp".to_string(),
                            Value::String(ts.as_str().to_string()),
                        );
                    }
                    if let Some(lvl) = caps.get(2) {
                        map.insert("level".to_string(), Value::String(lvl.as_str().to_string()));
                    }
                    if let Some(msg) = caps.get(3) {
                        map.insert(
                            "message".to_string(),
                            Value::String(msg.as_str().to_string()),
                        );
                    }
                }
                items.push(Value::Object(map));
            }
            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| format!("JSON serialization error: {}", e))?;
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("sqlite", "json") | ("db", "json") => {
            let conn =
                Connection::open_with_flags(input, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .map_err(|e| format!("Failed to open SQLite: {}", e))?;
            conn.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA query_only=ON;")
                .map_err(|e| e.to_string())?;
            let mut table_names = Vec::new();
            let mut stmt = conn
                .prepare("SELECT name FROM sqlite_master WHERE type='table'")
                .map_err(|e| format!("SQL error: {}", e))?;
            let rows = stmt
                .query_map([], |row| row.get::<_, String>(0))
                .map_err(|e| format!("SQL error: {}", e))?;
            for name in rows {
                table_names.push(name.map_err(|e| format!("SQL error: {}", e))?);
            }

            let mut db_dump = serde_json::Map::new();
            for table in table_names {
                let mut table_data = Vec::new();
                let query = format!("SELECT * FROM \"{}\"", table.replace('"', "\"\""));
                let mut stmt = conn
                    .prepare(&query)
                    .map_err(|e| format!("SQL error: {}", e))?;
                let column_names: Vec<String> =
                    stmt.column_names().iter().map(|s| s.to_string()).collect();
                let mut rows = stmt.query([]).map_err(|e| format!("SQL error: {}", e))?;
                while let Some(row) = rows.next().map_err(|e| format!("SQL error: {}", e))? {
                    let mut row_map = serde_json::Map::new();
                    for (i, col) in column_names.iter().enumerate() {
                        let val: Value = match row.get_ref(i) {
                            Ok(rusqlite::types::ValueRef::Null) => Value::Null,
                            Ok(rusqlite::types::ValueRef::Integer(n)) => Value::Number(n.into()),
                            Ok(rusqlite::types::ValueRef::Real(f)) => {
                                serde_json::Number::from_f64(f)
                                    .map(Value::Number)
                                    .unwrap_or(Value::Null)
                            }
                            Ok(rusqlite::types::ValueRef::Text(t)) => {
                                Value::String(String::from_utf8_lossy(t).into_owned())
                            }
                            Ok(rusqlite::types::ValueRef::Blob(b)) => {
                                Value::String(base64::encode(b))
                            }
                            Err(_) => Value::Null,
                        };
                        row_map.insert(col.clone(), val);
                    }
                    table_data.push(Value::Object(row_map));
                }
                db_dump.insert(table, Value::Array(table_data));
            }
            let json = serde_json::to_string_pretty(&db_dump)
                .map_err(|e| format!("JSON serialization error: {}", e))?;
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("bib", "json") => {
            let content =
                std::fs::read_to_string(input).map_err(|e| format!("Read BibTeX error: {}", e))?;
            let entry_re = regex::Regex::new(r"@(\w+)\s*\{\s*([^,]+),([\s\S]*?)\}")
                .map_err(|e| format!("Regex error: {}", e))?;
            let field_re = regex::Regex::new(r#"(\w+)\s*=\s*[\{"]?([\s\S]*?)[\}"]?(?:,|$)"#)
                .map_err(|e| format!("Regex error: {}", e))?;

            let mut entries = Vec::new();
            for caps in entry_re.captures_iter(&content) {
                let mut map = serde_json::Map::new();
                map.insert("type".to_string(), Value::String(caps[1].to_string()));
                map.insert("id".to_string(), Value::String(caps[2].trim().to_string()));
                let fields_str = &caps[3];
                for f_caps in field_re.captures_iter(fields_str) {
                    let key = f_caps[1].to_lowercase();
                    if key == "type" || key == "id" {
                        continue;
                    }
                    let cleaned = f_caps[2]
                        .trim()
                        .trim_matches(|c| c == '{' || c == '}' || c == '"')
                        .to_string();
                    map.insert(key, Value::String(cleaned));
                }
                entries.push(Value::Object(map));
            }
            let json =
                serde_json::to_string_pretty(&entries).map_err(|e| format!("JSON error: {}", e))?;
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("ics", "json") => {
            let content =
                std::fs::read_to_string(input).map_err(|e| format!("Read ICS error: {}", e))?;
            let mut events = Vec::new();
            let mut current_event = None;
            for line in content.lines() {
                let line = line.trim();
                if line == "BEGIN:VEVENT" {
                    current_event = Some(serde_json::Map::new());
                } else if line == "END:VEVENT" {
                    if let Some(ev) = current_event.take() {
                        events.push(Value::Object(ev));
                    }
                } else if let Some(ev) = current_event.as_mut() {
                    if let Some((key, val)) = line.split_once(':') {
                        ev.insert(key.to_lowercase(), Value::String(val.to_string()));
                    }
                }
            }
            let json =
                serde_json::to_string_pretty(&events).map_err(|e| format!("JSON error: {}", e))?;
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        _ => {
            return Err(format!(
                "Unsupported data conversion: {} to {}",
                input_ext, target_ext
            ))
        }
    }

    Ok(())
}

fn validate_headers<'a>(headers: impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for header in headers {
        if header.is_empty() || !seen.insert(header) {
            return Err("Empty or duplicate column names would lose data".into());
        }
    }
    Ok(())
}

fn safe_csv_cell(value: &str) -> std::borrow::Cow<'_, str> {
    if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        std::borrow::Cow::Owned(format!("'{value}"))
    } else {
        std::borrow::Cow::Borrowed(value)
    }
}

fn check_xlsx_size(path: &Path) -> Result<(), String> {
    let mut zip = zip::ZipArchive::new(File::open(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if zip.len() > 10_000 {
        return Err("XLSX contains too many entries".into());
    }
    let mut total = 0u64;
    for i in 0..zip.len() {
        total = total
            .checked_add(zip.by_index(i).map_err(|e| e.to_string())?.size())
            .ok_or("XLSX size overflow")?;
        if total > 128 * 1024 * 1024 {
            return Err("Expanded XLSX exceeds 128 MiB safety limit".into());
        }
    }
    Ok(())
}

fn validate_value(value: &Value, depth: usize) -> Result<(), String> {
    if depth > 64 {
        return Err("Structured data exceeds 64 nesting levels".into());
    }
    match value {
        Value::Array(values) => {
            for value in values {
                validate_value(value, depth + 1)?;
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                validate_value(value, depth + 1)?;
            }
        }
        _ => (),
    }
    Ok(())
}

fn validate_xml(value: &Value) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            let mut tags = std::collections::HashSet::new();
            for (key, value) in map {
                let tag = sanitize_xml_tag(key);
                if !tag.is_ascii() || !tags.insert(tag) {
                    return Err(
                        "XML field names are ambiguous or unsupported after normalization".into(),
                    );
                }
                validate_xml(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                validate_xml(value)?;
            }
        }
        Value::String(s)
            if s.chars().any(|c| {
                (c < ' ' && !matches!(c, '\t' | '\n' | '\r'))
                    || matches!(c, '\u{fffe}' | '\u{ffff}')
            }) =>
        {
            return Err("Text contains characters forbidden in XML 1.0".into())
        }
        _ => (),
    }
    Ok(())
}

fn parse_inferred_scalar(s: &str) -> Value {
    let trimmed = s.trim();
    if trimmed != s || (trimmed.len() > 1 && trimmed.starts_with('0') && !trimmed.starts_with("0."))
    {
        return Value::String(s.into());
    }
    if trimmed.is_empty() {
        return Value::Null;
    }
    if trimmed.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if let Ok(i) = trimmed.parse::<i64>() {
        return Value::Number(i.into());
    }
    if let Ok(f) = trimmed.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(f) {
            return Value::Number(num);
        }
    }
    Value::String(s.to_string())
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn sanitize_xml_tag(name: &str) -> String {
    let mut safe = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();
    if safe.is_empty() {
        return "item".to_string();
    }
    // XML element names cannot start with a digit or hyphen
    if safe.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
        safe = format!("item_{}", safe);
    }
    safe
}

fn val_to_xml(v: &Value, b: &mut String, depth: usize) {
    let indent = "  ".repeat(depth);
    match v {
        Value::Object(map) => {
            for (k, val) in map {
                let tag = sanitize_xml_tag(k);
                if val.is_object() || val.is_array() {
                    b.push_str(&format!("{}<{}>\n", indent, tag));
                    val_to_xml(val, b, depth + 1);
                    b.push_str(&format!("{}</{}>\n", indent, tag));
                } else {
                    b.push_str(&format!("{}<{}>", indent, tag));
                    val_to_xml(val, b, 0);
                    b.push_str(&format!("</{}>\n", tag));
                }
            }
        }
        Value::Array(arr) => {
            for val in arr {
                b.push_str(&format!("{}<item>\n", indent));
                val_to_xml(val, b, depth + 1);
                b.push_str(&format!("{}</item>\n", indent));
            }
        }
        Value::String(s) => {
            b.push_str(&escape_xml(s));
        }
        Value::Null => {}
        _ => {
            b.push_str(&escape_xml(&v.to_string()));
        }
    }
}

#[cfg(test)]
async fn run_archive_conversion(
    input: &Path,
    output: &Path,
    from: &str,
    to: &str,
) -> Result<(), String> {
    crate::archive::convert(input, output, from, to, None)
}

#[cfg(test)]
async fn run_data_conversion(
    input: &Path,
    output: &Path,
    from: &str,
    to: &str,
) -> Result<(), String> {
    run_data_conversion_sync(input, output, from, to)
}

#[cfg(test)]
use std::io::Write;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_csv_to_json_typed() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.csv");
        let output_path = dir.path().join("test.json");

        fs::write(
            &input_path,
            "name,age,active,score\nAlice,30,true,98.5\nBob,25,false,88.0",
        )
        .unwrap();

        run_data_conversion(&input_path, &output_path, "csv", "json")
            .await
            .unwrap();

        let output_content = fs::read_to_string(output_path).unwrap();
        let json: Value = serde_json::from_str(&output_content).unwrap();

        assert!(json.is_array());
        assert_eq!(json[0]["name"], "Alice");
        assert_eq!(json[0]["age"], 30);
        assert_eq!(json[0]["active"], true);
        assert_eq!(json[0]["score"], 98.5);
    }

    #[tokio::test]
    async fn test_json_to_csv_roundtrip() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.json");
        let output_path = dir.path().join("test.csv");

        fs::write(
            &input_path,
            r#"[{"a": 1, "b": "hello"}, {"a": 2, "b": "world"}]"#,
        )
        .unwrap();

        run_data_conversion(&input_path, &output_path, "json", "csv")
            .await
            .unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("a,b"));
        assert!(content.contains("1,hello"));
        assert!(content.contains("2,world"));
    }

    #[tokio::test]
    async fn test_json_to_xml_valid_tags() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.json");
        let output_path = dir.path().join("test.xml");

        fs::write(
            &input_path,
            r#"{"123numeric": "val", "safe_tag": "ok & <ready>"}"#,
        )
        .unwrap();

        run_data_conversion(&input_path, &output_path, "json", "xml")
            .await
            .unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("<item_123numeric>val</item_123numeric>"));
        assert!(content.contains("<safe_tag>ok &amp; &lt;ready&gt;</safe_tag>"));
    }

    #[tokio::test]
    async fn test_csv_to_sql_safe_identifiers() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("user-table.csv");
        let output_path = dir.path().join("test.sql");

        fs::write(&input_path, "id,user name\n1,O'Connor\n2,Smith").unwrap();

        run_data_conversion(&input_path, &output_path, "csv", "sql")
            .await
            .unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("CREATE TABLE IF NOT EXISTS \"user_table\""));
        assert!(content.contains("'O''Connor'"));
    }

    #[tokio::test]
    async fn test_tar_to_zip_safe_unpack() {
        let dir = tempdir().unwrap();
        let tar_path = dir.path().join("test.tar");
        let zip_path = dir.path().join("test.zip");

        // Create a real tar archive
        {
            let file = File::create(&tar_path).unwrap();
            let mut builder = tar::Builder::new(file);
            let mut header = tar::Header::new_gnu();
            header.set_path("hello.txt").unwrap();
            header.set_size(11);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, "hello world".as_bytes()).unwrap();
            builder.finish().unwrap();
        }

        run_archive_conversion(&tar_path, &zip_path, "tar", "zip")
            .await
            .unwrap();

        assert!(zip_path.exists());
        let zip_file = File::open(&zip_path).unwrap();
        let mut zip_arch = zip::ZipArchive::new(zip_file).unwrap();
        assert_eq!(zip_arch.len(), 1);
        let mut entry = zip_arch.by_name("hello.txt").unwrap();
        let mut s = String::new();
        entry.read_to_string(&mut s).unwrap();
        assert_eq!(s, "hello world");
    }

    #[tokio::test]
    async fn test_zip_slip_prevention() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("malicious.zip");
        let extract_dir = dir.path().join("extracted");

        {
            let file = File::create(&zip_path).unwrap();
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            // In modern zip crate, writing "../evil.txt" is caught or sanitized
            let _ = zip.start_file("../evil.txt", options);
            let _ = zip.write_all(b"attack");
            let _ = zip.finish();
        }

        let _result = run_archive_conversion(&zip_path, &extract_dir, "zip", "folder").await;
        // Either zip error or zip slip detected
        let evil_file = dir.path().join("evil.txt");
        assert!(
            !evil_file.exists(),
            "Zip slip vulnerability! File extracted outside destination"
        );
    }
}
