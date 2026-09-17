use crate::errors::map_technical_error;
use crate::formats::{get_category_for_extension, FileCategory};
use crate::sidecar::get_binary_command;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::{Path, PathBuf};
use calamine::{Reader, Xlsx};
use rust_xlsxwriter::Workbook;
use rusqlite::Connection;
use std::time::Instant;
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
    let output = get_binary_command(app, "ffprobe")
        .await?
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
            path,
        ])
        .output()
        .await
        .map_err(|e| format!("ffprobe execution error: {}", e))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let json: Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let streams = json["streams"].as_array().ok_or("No streams found in file")?;
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
    let height = video_stream
        .and_then(|s| s["height"].as_u64())
        .unwrap_or(0) as u32;

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

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let category = get_category_for_extension(&ext);
    let target_cat = get_category_for_extension(target_format);

    let mut warnings = Vec::new();

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
                    warnings.push(
                        "Input has no video stream. Output will be audio-only if supported."
                            .to_string(),
                    );
                }

                return ValidationResult {
                    is_valid: true,
                    warnings,
                    error: None,
                    file_info: Some(info),
                };
            }
            Err(e) => {
                // ffprobe failed, but maybe it's not a media file or ffprobe is missing
                warnings.push(format!("Could not probe file details: {}", e));
            }
        }
    }

    // Generic validation
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

    let input_ext = input_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect::<String>();
    if stem.is_empty() {
        return Err("Invalid input filename stem".to_string());
    }
    let out_dir = match &req.output_dir {
        Some(d) => PathBuf::from(d),
        None => input_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf(),
    };

    let target_ext = req.target_format.to_lowercase();
    if !target_ext.chars().all(|c| c.is_alphanumeric()) {
        return Err(format!("Invalid target format: {}", req.target_format));
    }
    let output_filename = format!("{}_converted.{}", stem, target_ext);
    let output_path = out_dir.join(output_filename);

    let start_time = Instant::now();
    let original_size = std::fs::metadata(input_path).map(|m| m.len()).unwrap_or(0);

    let category = get_category_for_extension(&input_ext);
    let target_category = get_category_for_extension(&target_ext);

    let execution_result = if input_ext == "pdf" {
        run_pdf_conversion(&app, input_path, &output_path, &target_ext).await.map(|_| vec![])
    } else if category == FileCategory::Data {
        run_data_conversion(input_path, &output_path, &input_ext, &target_ext).await.map(|_| vec![])
    } else if category == FileCategory::Archive {
        run_archive_conversion(input_path, &output_path, &input_ext, &target_ext).await.map(|_| vec![])
    } else if category == FileCategory::Image || category == FileCategory::Vector {
        // Determine if target is also image/vector (ImageMagick) or document (Pandoc fallback)
        let target_category = get_category_for_extension(&target_ext);
        if target_category == FileCategory::Document {
            run_pandoc_conversion(&app, input_path, &output_path).await.map(|_| vec![])
        } else {
            run_image_magick_conversion(&app, input_path, &output_path).await.map(|_| vec![])
        }
    } else if target_category == FileCategory::Document {
        run_pandoc_conversion(&app, input_path, &output_path).await.map(|_| vec![])
    } else {
        run_ffmpeg_conversion(&app, &req, input_path, &output_path, &target_ext).await
    };

    let elapsed = start_time.elapsed().as_millis() as u64;

    match execution_result {
        Ok(warnings) => {
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
                warnings,
            })
        }
        Err(err) => {
            let friendly_err = map_technical_error(&err, &req.input_path, &req.target_format);
            Ok(ConversionResult {
                job_id: Uuid::new_v4().to_string(),
                input_path: req.input_path.clone(),
                output_path: output_path.to_string_lossy().to_string(),
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

async fn run_pdf_conversion<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    input: &Path,
    output: &Path,
    target_ext: &str,
) -> Result<(), String> {
    let out_dir = output.parent().unwrap_or_else(|| Path::new("."));

    if target_ext == "txt" {
        let text_path = out_dir.join(format!(".file2file-{}.txt", Uuid::new_v4()));
        let input_str = input.to_str().ok_or("Input path contains invalid UTF-8")?;
        let text_path_str = text_path
            .to_str()
            .ok_or("Temporary path contains invalid UTF-8")?;

        let extracted = get_binary_command(app, "pdftotext")
            .await?
            .args(["-layout", input_str, text_path_str])
            .output()
            .await
            .map_err(|e| {
                format!(
                    "pdftotext execution error: {}. PDF conversion requires Poppler's pdftotext utility.",
                    e
                )
            })?;

        if !extracted.status.success() {
            let _ = std::fs::remove_file(&text_path);
            return Err(String::from_utf8_lossy(&extracted.stderr)
                .trim()
                .to_string());
        }

        let result = std::fs::copy(&text_path, output)
            .map(|_| ())
            .map_err(|e| format!("Could not save extracted text: {e}"));

        let _ = std::fs::remove_file(&text_path);
        result
    } else {
        // High-fidelity conversion for DOCX, HTML, ODT, etc.
        let input_str = input.to_str().ok_or("Input path contains invalid UTF-8")?;
        let out_dir = output.parent().unwrap_or_else(|| Path::new("."));
        let temp_prefix = out_dir.join(format!(".file2file-{}", Uuid::new_v4()));
        let temp_prefix_str = temp_prefix
            .to_str()
            .ok_or("Temporary path contains invalid UTF-8")?;

        let output_gen = get_binary_command(app, "pdftohtml")
            .await?
            .args(["-c", "-dataurls", "-noframes", input_str, temp_prefix_str])
            .output()
            .await
            .map_err(|e| {
                format!(
                    "pdftohtml execution error: {}. PDF conversion requires Poppler's pdftohtml utility.",
                    e
                )
            })?;

        if !output_gen.status.success() {
            return Err(String::from_utf8_lossy(&output_gen.stderr).to_string());
        }

        let temp_html = out_dir.join(format!("{}.html", temp_prefix.file_name().unwrap().to_str().unwrap()));

        if !temp_html.exists() {
            return Err("pdftohtml failed to generate temporary HTML file.".to_string());
        }

        let result = run_pandoc_conversion(app, &temp_html, output).await;

        let _ = std::fs::remove_file(&temp_html);
        result
    }
}

async fn run_image_magick_conversion<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    input: &Path,
    output: &Path,
) -> Result<(), String> {
    let input_str = input.to_str().ok_or("Input path contains invalid UTF-8")?;
    let output_str = output.to_str().ok_or("Output path contains invalid UTF-8")?;

    let output_ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut args = Vec::new();

    // If it's an animated format, coalesce frames first
    let input_ext = input.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if input_ext == "gif" || input_ext == "webp" {
        args.push("-coalesce".to_string());
    }

    args.push(input_str.to_string());

    // Transparency preservation logic
    if output_ext == "jpg" || output_ext == "jpeg" {
        // JPG doesn't support alpha, so flatten against white to avoid black silhouettes
        args.push("-background".to_string());
        args.push("white".to_string());
        args.push("-flatten".to_string());
    }

    args.push(output_str.to_string());

    let result = get_binary_command(app, "magick")
        .await?
        .args(args)
        .output()
        .await
        .map_err(|e| {
            format!("magick execution error: {}. Image conversion requires ImageMagick (magick) to be installed.", e)
        })?;

    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).trim().to_string())
    }
}

async fn run_ffmpeg_conversion<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    req: &ConversionRequest,
    input: &Path,
    output: &Path,
    target_ext: &str,
) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    if req.hardware_accel {
        let hw_res = execute_ffmpeg(app, req, input, output, target_ext, true).await;
        if hw_res.is_ok() {
            return Ok(warnings);
        }
        // Fallback occurred
        warnings.push("Hardware acceleration failed, fell back to CPU encoding.".to_string());
    }
    execute_ffmpeg(app, req, input, output, target_ext, false).await.map(|_| warnings)
}

async fn execute_ffmpeg<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    req: &ConversionRequest,
    input: &Path,
    output: &Path,
    target_ext: &str,
    use_hw: bool,
) -> Result<(), String> {
    let input_str = input.to_str().ok_or("Input path contains invalid UTF-8")?;
    let output_str = output.to_str().ok_or("Output path contains invalid UTF-8")?;

    let mut args = vec!["-y".to_string(), "-i".to_string(), input_str.to_string()];

    // Add universal stream and metadata mapping as default
    // We will override this for specific formats like audio-only or image extraction
    let mut map_all = true;

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
            // Ensure we copy subtitles where possible
            args.push("-c:s".to_string());
            args.push("copy".to_string());
        }
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "opus" => {
            args.push("-vn".to_string());
            map_all = false; // We only want audio
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
        "webp" | "png" | "jpg" | "jpeg" | "avif" | "gif" => {
            map_all = false; // Extraction
            args.push("-c:v".to_string());
            args.push("-c:a".to_string());
            args.push("aac".to_string());
            match target_ext {
                "ico" => {
                    // ImageMagick is better for ICO multi-res
                    run_image_magick_conversion(app, input, output).await?;
                    return Ok(());
                }
                "webp" => {
                    args.push("libwebp".to_string());
                }
                "avif" => {
                    args.push("libaom-av1".to_string());
                }
                "gif" => {
                    args.push("fps=15,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse".to_string());
                }
                "tiff" => {
                    args.push("tiff".to_string());
                }
                "bmp" => {
                    args.push("bmp".to_string());
                }
                "jpeg" => {
                    args.push("jpeg".to_string());
                }
                "heic" => {
                    args.push("libhevc".to_string());
                }
                "png" => {
                    args.push("png".to_string());
                }
                _ => {}
            }
        }
        "srt" => {
            map_all = false;
            args.push("-vn".to_string());
            args.push("-c:s".to_string());
            args.push("-map".to_string());
            args.push("0:s:0".to_string());
        }
        "raw" | "cr2" | "nef" | "arw" | "dng" => {
            map_all = false;
            args.push("-vn".to_string());
            args.push("-c:v".to_string());
            args.push("libjpeg".to_string());
        }
        _ => {}
    }

    if map_all {
        args.push("-map".to_string());
        args.push("0".to_string());
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
    } else {
        // Explicitly map all metadata and chapters from the first input
        args.push("-map_metadata".to_string());
        args.push("0".to_string());
        args.push("-map_chapters".to_string());
        args.push("0".to_string());
    }

    args.push(output_str.to_string());

    let output = get_binary_command(app, "ffmpeg")
        .await?
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

async fn run_pandoc_conversion<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    input: &Path,
    output: &Path,
) -> Result<(), String> {
    let input_str = input.to_str().ok_or("Input path contains invalid UTF-8")?;
    let output_str = output.to_str().ok_or("Output path contains invalid UTF-8")?;
    let output_ext = output
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mut args = vec![input_str.to_string(), "-o".to_string(), output_str.to_string()];

    // Add high-fidelity math support by default if input/output likely has it
    args.push("-f".to_string());
    args.push("html+tex_math_dollars".to_string());

    // Universal image preservation and layout flags
    if output_ext == "html" || output_ext == "htm" {
        args.push("--embed-resources".to_string());
        args.push("--standalone".to_string());
    } else if output_ext == "docx" || output_ext == "odt" || output_ext == "epub" {
        // Pandoc embeds images by default for these, but we ensure it knows how to handle media
        // if the source has external images (though we use dataurls for PDF path)
        args.push("--extract-media=.".to_string());
    }

    let output = get_binary_command(app, "pandoc")
        .await?
        .args(args)
        .output()
        .await
        .map_err(|e| format!("Pandoc execution error: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

async fn run_data_conversion(
    input: &Path,
    output: &Path,
    input_ext: &str,
    target_ext: &str,
) -> Result<(), String> {
    let input_file = File::open(input).map_err(|e| format!("Failed to open input file: {}", e))?;
    let reader = BufReader::new(input_file);

    // Helper to get a JSON Value from various input formats
    let value: Value = match input_ext {
        "json" => serde_json::from_reader(reader).map_err(|e| format!("JSON parse error: {}", e))?,
        "csv" => {
            let mut csv_reader = csv::Reader::from_reader(reader);
            let headers = csv_reader.headers().map_err(|e| format!("CSV headers error: {}", e))?.clone();
            let mut items = Vec::new();
            for result in csv_reader.records() {
                let record = result.map_err(|e| format!("CSV record error: {}", e))?;
                let mut map = serde_json::Map::new();
                for (header, value) in headers.iter().zip(record.iter()) {
                    map.insert(header.to_string(), Value::String(value.to_string()));
                }
                items.push(Value::Object(map));
            }
            Value::Array(items)
        }
        "yaml" => serde_yaml::from_reader(reader).map_err(|e| format!("YAML parse error: {}", e))?,
        "toml" => {
            let content = std::fs::read_to_string(input).map_err(|e| format!("Failed to read input file: {}", e))?;
            toml::from_str(&content).map_err(|e| format!("TOML parse error: {}", e))?
        }
        "xml" => {
            let content = std::fs::read_to_string(input).map_err(|e| format!("Failed to read input file: {}", e))?;
            quick_xml::de::from_str(&content).map_err(|e| format!("XML parse error: {}", e))?
        }
        _ => {
            // Fallback for formats that might be handled differently below (like xlsx)
            Value::Null
        }
    };

    match (input_ext, target_ext) {
        (i, "json") if i != "json" && value != Value::Null => {
            let json = serde_json::to_string_pretty(&value).map_err(|e| format!("JSON serialization error: {}", e))?;
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
            return Ok(());
        }
        (i, "yaml") if i != "yaml" && value != Value::Null => {
            let yaml = serde_yaml::to_string(&value).map_err(|e| format!("YAML serialization error: {}", e))?;
            std::fs::write(output, yaml).map_err(|e| format!("Failed to write output: {}", e))?;
            return Ok(());
        }
        (i, "toml") if i != "toml" && value != Value::Null => {
            let toml_string = toml::to_string(&value).map_err(|e| format!("TOML serialization error: {}", e))?;
            std::fs::write(output, toml_string).map_err(|e| format!("Failed to write output: {}", e))?;
            return Ok(());
        }
        (i, "csv") if i != "csv" && value != Value::Null => {
            if let Some(arr) = value.as_array() {
                if arr.is_empty() { return Ok(()); }
                let mut headers = std::collections::BTreeSet::new();
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        for key in obj.keys() { headers.insert(key.to_string()); }
                    }
                }
                let header_list: Vec<String> = headers.into_iter().collect();
                let mut wtr = csv::Writer::from_path(output).map_err(|e| format!("Failed to create CSV writer: {}", e))?;
                wtr.write_record(&header_list).map_err(|e| format!("CSV header write error: {}", e))?;
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        let row: Vec<String> = header_list.iter().map(|h| {
                            obj.get(h).and_then(|v| match v {
                                Value::String(s) => Some(s.clone()),
                                Value::Null => Some("".to_string()),
                                _ => Some(v.to_string()),
                            }).unwrap_or_default()
                        }).collect();
                        wtr.write_record(&row).map_err(|e| format!("CSV row write error: {}", e))?;
                    }
                }
                wtr.flush().map_err(|e| format!("Failed to flush CSV: {}", e))?;
            } else {
                return Err("Input must be an array of objects to convert to CSV".to_string());
            }
            return Ok(());
        }
        ("csv", "sql") => {
            // Already handled by generic path above if we wanted json,
            // but sql needs special handling
            let mut csv_reader = csv::Reader::from_reader(File::open(input).unwrap());
            let headers = csv_reader.headers().map_err(|e| format!("CSV headers error: {}", e))?.clone();
            let table_name = input.file_stem().and_then(|s| s.to_str()).unwrap_or("imported_table")
                .chars().filter(|c| c.is_alphanumeric() || *c == '_').collect::<String>();
            let mut sql = format!("CREATE TABLE IF NOT EXISTS `{}` (\n", table_name);
            for (i, h) in headers.iter().enumerate() {
                let safe_h = h.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect::<String>();
                sql.push_str(&format!("  `{}` TEXT{}", safe_h, if i < headers.len() - 1 { "," } else { "" }));
                sql.push('\n');
            }
            sql.push_str(");\n\n");
            for result in csv_reader.records() {
                let record = result.map_err(|e| format!("CSV record error: {}", e))?;
                sql.push_str(&format!("INSERT INTO `{}` VALUES (", table_name));
                for (i, val) in record.iter().enumerate() {
                    sql.push_str(&format!("'{}'{}", val.replace('\'', "''"), if i < record.len() - 1 { ", " } else { "" }));
                }
                sql.push_str(");\n");
            }
            std::fs::write(output, sql).map_err(|e| format!("Failed to write output: {}", e))?;
            return Ok(());
        }
        ("log", "json") => {
            let mut content = String::new();
            File::open(input).map_err(|e| format!("Open log error: {}", e))?.read_to_string(&mut content)
                .map_err(|e| format!("Read log error: {}", e))?;
            let mut items = Vec::new();
            for line in content.lines() {
                if line.trim().is_empty() { continue; }
                let mut map = serde_json::Map::new();
                map.insert("raw".to_string(), Value::String(line.to_string()));
                // Basic parsing attempt: look for timestamp or level
                if let Some(caps) = regex::Regex::new(r"(\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})\s+(\w+)\s+(.*)").unwrap().captures(line) {
                    map.insert("timestamp".to_string(), Value::String(caps[1].to_string()));
                    map.insert("level".to_string(), Value::String(caps[2].to_string()));
                    map.insert("message".to_string(), Value::String(caps[3].to_string()));
                }
                items.push(Value::Object(map));
            }
            let json = serde_json::to_string_pretty(&items).unwrap();
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("sqlite", "json") | ("db", "json") => {
            let conn = Connection::open(input).map_err(|e| format!("Failed to open SQLite: {}", e))?;
            let mut table_names = Vec::new();
            let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table'").map_err(|e| format!("SQL error: {}", e))?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0)).map_err(|e| format!("SQL error: {}", e))?;
            for name in rows {
                table_names.push(name.map_err(|e| format!("SQL error: {}", e))?);
            }

            let mut db_dump = serde_json::Map::new();
            for table in table_names {
                let mut table_data = Vec::new();
                let mut stmt = conn.prepare(&format!("SELECT * FROM {}", table)).map_err(|e| format!("SQL error: {}", e))?;
                let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();
                let mut rows = stmt.query([]).map_err(|e| format!("SQL error: {}", e))?;
                while let Some(row) = rows.next().map_err(|e| format!("SQL error: {}", e))? {
                    let mut row_map = serde_json::Map::new();
                    for (i, col) in column_names.iter().enumerate() {
                        let val: Value = match row.get_ref(i).unwrap() {
                            rusqlite::types::ValueRef::Null => Value::Null,
                            rusqlite::types::ValueRef::Integer(i) => Value::Number(i.into()),
                            rusqlite::types::ValueRef::Real(f) => serde_json::Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null),
                            rusqlite::types::ValueRef::Text(t) => Value::String(String::from_utf8_lossy(t).into_owned()),
                            rusqlite::types::ValueRef::Blob(b) => Value::String(base64::encode(b)),
                        };
                        row_map.insert(col.clone(), val);
                    }
                    table_data.push(Value::Object(row_map));
                }
                db_dump.insert(table, Value::Array(table_data));
            }
            let json = serde_json::to_string_pretty(&db_dump).unwrap();
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("bib", "json") => {
            let content = std::fs::read_to_string(input).map_err(|e| format!("Read BibTeX error: {}", e))?;
            let mut entries = Vec::new();
            let entry_re = regex::Regex::new(r"@(\w+)\s*\{\s*([^,]+),([\s\S]*?)\}").unwrap();
            for caps in entry_re.captures_iter(&content) {
                let mut map = serde_json::Map::new();
                map.insert("type".to_string(), Value::String(caps[1].to_string()));
                map.insert("id".to_string(), Value::String(caps[2].trim().to_string()));
                let fields_str = &caps[3];
                // Simplify field regex: key = {val} or key = "val" or key = val
                let field_re = regex::Regex::new(r#"(\w+)\s*=\s*[\{"]?([\s\S]*?)[\}"]?(?:,|$)"#).unwrap();
                for f_caps in field_re.captures_iter(fields_str) {
                    let key = f_caps[1].to_lowercase();
                    if key == "type" || key == "id" { continue; }
                    map.insert(key, Value::String(f_caps[2].trim().trim_matches(|c| c == '{' || c == '}' || c == '"').to_string()));
                }
                entries.push(Value::Object(map));
            }
            let json = serde_json::to_string_pretty(&entries).unwrap();
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("ics", "json") => {
            let content = std::fs::read_to_string(input).map_err(|e| format!("Read ICS error: {}", e))?;
            let mut events = Vec::new();
            let mut current_event = None;
            for line in content.lines() {
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
            let json = serde_json::to_string_pretty(&events).unwrap();
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

async fn run_archive_conversion(
    input: &Path,
    output: &Path,
    input_ext: &str,
    target_ext: &str,
) -> Result<(), String> {
    match (input_ext, target_ext) {
        (_, "zip") => {
            let file =
                File::create(output).map_err(|e| format!("Failed to create zip file: {}", e))?;
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o755);

            if input.is_dir() {
                add_dir_to_zip(&mut zip, input, input, options)?;
            } else {
                let name = input
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or("Invalid filename for zip")?;
                add_file_to_zip(&mut zip, input, name, options)?;
            }
            zip.finish()
                .map_err(|e| format!("Failed to finish zip: {}", e))?;
        }
        ("zip", _) => {
            // Extraction
            let file = File::open(input).map_err(|e| format!("Failed to open zip file: {}", e))?;
            let mut archive =
                zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip archive: {}", e))?;
            let out_dir = output.with_extension("");
            fs::create_dir_all(&out_dir)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;

            let mut total_extracted_size: u64 = 0;
            let max_total_size: u64 = 5 * 1024 * 1024 * 1024; // 5GB limit for safety
            let max_files = 10000;

            if archive.len() > max_files {
                return Err(format!("Zip archive contains too many files (max {})", max_files));
            }

            for i in 0..archive.len() {
                let mut file = archive
                    .by_index(i)
                    .map_err(|e| format!("Failed to read zip entry: {}", e))?;

                total_extracted_size += file.size();
                if total_extracted_size > max_total_size {
                    return Err("Zip extraction exceeded safety size limit (5GB)".to_string());
                }

                let outpath = match file.enclosed_name() {
                    Some(path) => out_dir.join(path),
                    None => continue,
                };

                if (*file.name()).ends_with('/') {
                    fs::create_dir_all(&outpath)
                        .map_err(|e| format!("Failed to create sub-directory: {}", e))?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(p)
                                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                        }
                    }
                    let mut outfile = File::create(&outpath).map_err(|e| {
                        format!("Failed to create output file {}: {}", outpath.display(), e)
                    })?;
                    io::copy(&mut file, &mut outfile)
                        .map_err(|e| format!("Failed to extract file: {}", e))?;
                }
            }
        }
        _ => {
            return Err(format!(
                "Unsupported archive conversion: {} to {}",
                input_ext, target_ext
            ))
        }
    }
    Ok(())
}

fn add_file_to_zip<W: io::Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    path: &Path,
    name: &str,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    zip.start_file(name, options)
        .map_err(|e| format!("Zip error: {}", e))?;
    let mut f = File::open(path).map_err(|e| format!("Failed to open file for zipping: {}", e))?;
    io::copy(&mut f, zip).map_err(|e| format!("Failed to write to zip: {}", e))?;
    Ok(())
}

fn add_dir_to_zip<W: io::Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    full_path: &Path,
    base_path: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    for entry in fs::read_dir(full_path).map_err(|e| format!("Failed to read directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        let name = path
            .strip_prefix(base_path)
            .map_err(|e| format!("Prefix error: {}", e))?
            .to_str()
            .ok_or("Path contains invalid UTF-8")?;

        if path.is_dir() {
            add_dir_to_zip(zip, &path, base_path, options)?;
        } else {
            add_file_to_zip(zip, &path, name, options)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_csv_to_json() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.csv");
        let output_path = dir.path().join("test.json");

        fs::write(&input_path, "name,age\nAlice,30\nBob,25").unwrap();

        run_data_conversion(&input_path, &output_path, "csv", "json")
            .await
            .unwrap();

        let output_content = fs::read_to_string(output_path).unwrap();
        let json: Value = serde_json::from_str(&output_content).unwrap();

        assert!(json.is_array());
        assert_eq!(json[0]["name"], "Alice");
        assert_eq!(json[1]["age"], "25");
    }

    #[tokio::test]
    async fn test_json_to_csv_mixed_keys() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.json");
        let output_path = dir.path().join("test.csv");

        // "a" is common, "b" only in first, "c" only in second
        fs::write(&input_path, r#"[{"a": 1, "b": 2}, {"a": 3, "c": 4}]"#).unwrap();

        run_data_conversion(&input_path, &output_path, "json", "csv")
            .await
            .unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        let mut lines = content.lines();

        // BTreeSet sorts keys: a, b, c
        assert_eq!(lines.next(), Some("a,b,c"));
        assert_eq!(lines.next(), Some("1,2,"));
        assert_eq!(lines.next(), Some("3,,4"));
    }

    #[tokio::test]
    async fn test_json_to_csv_empty() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.json");
        let output_path = dir.path().join("test.csv");

        fs::write(&input_path, "[]").unwrap();

        run_data_conversion(&input_path, &output_path, "json", "csv")
            .await
            .unwrap();

        // Output file might not even be created if it's empty, or it's empty
        if output_path.exists() {
            let content = fs::read_to_string(output_path).unwrap();
            assert!(content.is_empty());
        }
    }

    #[tokio::test]
    async fn test_zip_unzip_roundtrip() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let zip_path = dir.path().join("test.zip");

        fs::write(&file_path, "hello world").unwrap();

        // Zip it
        run_archive_conversion(&file_path, &zip_path, "txt", "zip")
            .await
            .unwrap();
        assert!(zip_path.exists());

        // Unzip it
        // The current implementation of zip to folder uses output.with_extension("")
        run_archive_conversion(&zip_path, &zip_path, "zip", "folder")
            .await
            .unwrap();

        let extracted_file = dir.path().join("test/test.txt");
        assert!(extracted_file.exists());
        let content = fs::read_to_string(extracted_file).unwrap();
        assert_eq!(content, "hello world");
    }

    #[tokio::test]
    async fn test_xml_to_json() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.xml");
        let output_path = dir.path().join("test.json");

        fs::write(&input_path, r#"<root><item id="1">Hello</item></root>"#).unwrap();

        run_data_conversion(&input_path, &output_path, "xml", "json")
            .await
            .unwrap();

        let output_content = fs::read_to_string(output_path).unwrap();
        let json: Value = serde_json::from_str(&output_content).unwrap();
        // quick-xml might deserialize as {"item": {"@id": "1", "$value": "Hello"}}
        // or just {"item": {"id": "1", "#text": "Hello"}} depending on version/config
        assert!(json.get("item").is_some());
    }

    #[tokio::test]
    async fn test_json_to_xml() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.json");
        let output_path = dir.path().join("test.xml");

        fs::write(&input_path, r#"{"name": "test", "value": 123}"#).unwrap();

        run_data_conversion(&input_path, &output_path, "json", "xml")
            .await
            .unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("<name>test</name>"));
        assert!(content.contains("<value>123</value>"));
    }

    #[tokio::test]
    async fn test_csv_to_sql() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("my_table.csv");
        let output_path = dir.path().join("test.sql");

        fs::write(&input_path, "id,name\n1,O'Reilly\n2,Bob").unwrap();

        run_data_conversion(&input_path, &output_path, "csv", "sql")
            .await
            .unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("CREATE TABLE IF NOT EXISTS `my_table`"));
        assert!(content.contains("INSERT INTO `my_table` VALUES ('1', 'O''Reilly');"));
    }

    #[tokio::test]
    async fn test_bib_to_json() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.bib");
        let output_path = dir.path().join("test.json");

        fs::write(&input_path, "@article{sample, author={John Doe}, title={A Paper}\n}").unwrap();

        run_data_conversion(&input_path, &output_path, "bib", "json")
            .await
            .unwrap();

        let content = fs::read_to_string(output_path).unwrap();
        let json: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(json[0]["type"], "article");
        assert!(json[0]["author"].as_str().unwrap().contains("John Doe"));
    }

    #[tokio::test]
    async fn test_invalid_target_format() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.txt");
        fs::write(&input_path, "test").unwrap();

        let app = tauri::test::mock_app();
        let req = ConversionRequest {
            input_path: input_path.to_str().unwrap().to_string(),
            output_dir: None,
            target_format: "../evil".to_string(),
            crf: None,
            resolution: None,
            hardware_accel: false,
            selected_encoder: None,
            strip_metadata: false,
            audio_bitrate: None,
        };
        let result = convert_single_file(app.handle().clone(), req).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("Invalid target format"),
            "Expected 'Invalid target format', got: {}",
            err
        );
    }
}
