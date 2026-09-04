use crate::formats::{get_category_for_extension, FileCategory};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, BufReader};
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
    } else if category == FileCategory::Data {
        run_data_conversion(input_path, &output_path, &input_ext, &target_ext).await
    } else if category == FileCategory::Archive {
        run_archive_conversion(input_path, &output_path, &input_ext, &target_ext).await
    } else if category == FileCategory::Image || category == FileCategory::Vector {
        run_image_magick_conversion(&app, input_path, &output_path).await
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
    let extracted = app
        .shell()
        .sidecar("pdftotext")
        .map_err(|e| format!("Failed to create pdftotext sidecar: {}", e))?
        .args(["-layout", input.to_str().unwrap(), text_path.to_str().unwrap()])
        .output()
        .await
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

async fn run_image_magick_conversion(
    app: &tauri::AppHandle,
    input: &Path,
    output: &Path,
) -> Result<(), String> {
    let result = app
        .shell()
        .sidecar("magick")
        .map_err(|e| format!("Failed to create ImageMagick sidecar: {}", e))?
        .args([input.to_str().unwrap(), output.to_str().unwrap()])
        .output()
        .await
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

async fn run_data_conversion(
    input: &Path,
    output: &Path,
    input_ext: &str,
    target_ext: &str,
) -> Result<(), String> {
    let input_file = File::open(input).map_err(|e| format!("Failed to open input file: {}", e))?;
    let reader = BufReader::new(input_file);

    match (input_ext, target_ext) {
        ("csv", "json") => {
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
            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| format!("JSON serialization error: {}", e))?;
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("json", "csv") => {
            let items: Value = serde_json::from_reader(reader)
                .map_err(|e| format!("JSON parse error: {}", e))?;

            if let Some(arr) = items.as_array() {
                if arr.is_empty() {
                    return Ok(());
                }

                // Collect all unique keys for headers
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

                // Write headers
                wtr.write_record(&header_list)
                    .map_err(|e| format!("CSV header write error: {}", e))?;

                // Write rows
                for item in arr {
                    if let Some(obj) = item.as_object() {
                        let row: Vec<String> = header_list.iter()
                            .map(|h| obj.get(h).and_then(|v| {
                                match v {
                                    Value::String(s) => Some(s.clone()),
                                    Value::Null => Some("".to_string()),
                                    _ => Some(v.to_string()),
                                }
                            }).unwrap_or_default())
                            .collect();
                        wtr.write_record(&row)
                            .map_err(|e| format!("CSV row write error: {}", e))?;
                    }
                }
                wtr.flush().map_err(|e| format!("Failed to flush CSV: {}", e))?;
            } else {
                return Err("JSON must be an array of objects to convert to CSV".to_string());
            }
        }
        ("json", "yaml") => {
            let value: Value = serde_json::from_reader(reader)
                .map_err(|e| format!("JSON parse error: {}", e))?;
            let yaml = serde_yaml::to_string(&value)
                .map_err(|e| format!("YAML serialization error: {}", e))?;
            std::fs::write(output, yaml).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("yaml", "json") => {
            let value: Value = serde_yaml::from_reader(reader)
                .map_err(|e| format!("YAML parse error: {}", e))?;
            let json = serde_json::to_string_pretty(&value)
                .map_err(|e| format!("JSON serialization error: {}", e))?;
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("json", "toml") => {
            let value: Value = serde_json::from_reader(reader)
                .map_err(|e| format!("JSON parse error: {}", e))?;
            let toml = toml::to_string_pretty(&value)
                .map_err(|e| format!("TOML serialization error: {}", e))?;
            std::fs::write(output, toml).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        ("toml", "json") => {
            let content = std::fs::read_to_string(input)
                .map_err(|e| format!("Failed to read input file: {}", e))?;
            let value: Value = toml::from_str(&content)
                .map_err(|e| format!("TOML parse error: {}", e))?;
            let json = serde_json::to_string_pretty(&value)
                .map_err(|e| format!("JSON serialization error: {}", e))?;
            std::fs::write(output, json).map_err(|e| format!("Failed to write output: {}", e))?;
        }
        _ => return Err(format!("Unsupported data conversion: {} to {}", input_ext, target_ext)),
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
            let file = File::create(output).map_err(|e| format!("Failed to create zip file: {}", e))?;
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated)
                .unix_permissions(0o755);

            if input.is_dir() {
                add_dir_to_zip(&mut zip, input, input, options)?;
            } else {
                add_file_to_zip(&mut zip, input, input.file_name().unwrap().to_str().unwrap(), options)?;
            }
            zip.finish().map_err(|e| format!("Failed to finish zip: {}", e))?;
        }
        ("zip", _) => {
            // Extraction
            let file = File::open(input).map_err(|e| format!("Failed to open zip file: {}", e))?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip archive: {}", e))?;
            let out_dir = output.with_extension("");
            fs::create_dir_all(&out_dir).map_err(|e| format!("Failed to create output directory: {}", e))?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| format!("Failed to read zip entry: {}", e))?;
                let outpath = match file.enclosed_name() {
                    Some(path) => out_dir.join(path),
                    None => continue,
                };

                if (*file.name()).ends_with('/') {
                    fs::create_dir_all(&outpath).map_err(|e| format!("Failed to create sub-directory: {}", e))?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(p).map_err(|e| format!("Failed to create parent directory: {}", e))?;
                        }
                    }
                    let mut outfile = File::create(&outpath).map_err(|e| format!("Failed to create output file {}: {}", outpath.display(), e))?;
                    io::copy(&mut file, &mut outfile).map_err(|e| format!("Failed to extract file: {}", e))?;
                }
            }
        }
        _ => return Err(format!("Unsupported archive conversion: {} to {}", input_ext, target_ext)),
    }
    Ok(())
}

fn add_file_to_zip<W: io::Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    path: &Path,
    name: &str,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    zip.start_file(name, options).map_err(|e| format!("Zip error: {}", e))?;
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
        let entry = entry.unwrap();
        let path = entry.path();
        let name = path.strip_prefix(base_path).unwrap().to_str().unwrap();

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

        run_data_conversion(&input_path, &output_path, "csv", "json").await.unwrap();

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

        run_data_conversion(&input_path, &output_path, "json", "csv").await.unwrap();

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

        run_data_conversion(&input_path, &output_path, "json", "csv").await.unwrap();

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
        run_archive_conversion(&file_path, &zip_path, "txt", "zip").await.unwrap();
        assert!(zip_path.exists());

        // Unzip it
        // The current implementation of zip to folder uses output.with_extension("")
        run_archive_conversion(&zip_path, &zip_path, "zip", "folder").await.unwrap();

        let extracted_file = dir.path().join("test/test.txt");
        assert!(extracted_file.exists());
        let content = fs::read_to_string(extracted_file).unwrap();
        assert_eq!(content, "hello world");
    }
}
