use file2file_lib::converter::{convert_single_file, ConversionRequest};
use file2file_lib::formats::{get_all_extensions, get_compatible_formats};
use file2file_lib::sidecar::check_sidecar_health;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_full_conversion_matrix() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    // Check engine health first
    let health = check_sidecar_health(handle).await;
    println!("Engine Health: {:?}", health);

    let extensions = get_all_extensions();
    let mut results = Vec::new();

    let test_files_dir = Path::new("../test_files");
    if !test_files_dir.exists() {
        fs::create_dir_all(test_files_dir).unwrap();
    }

    let output_dir = Path::new("../matrix_test_output");
    if output_dir.exists() {
        fs::remove_dir_all(output_dir).unwrap();
    }
    fs::create_dir_all(output_dir).unwrap();

    for input_ext in extensions {
        let sample_file = test_files_dir.join(format!("test.{}", input_ext));

        // Ensure sample file exists
        if !sample_file.exists() {
            create_minimal_valid_file(&sample_file, &input_ext);
        }

        if !sample_file.exists() {
            println!("Skipping matrix for .{} (no sample file created)", input_ext);
            continue;
        }

        let targets = get_compatible_formats(&input_ext);
        for target in targets {
            let target_ext = target.extension;
            println!("Testing: {} -> {}", input_ext, target_ext);

            let req = ConversionRequest {
                input_path: sample_file.to_string_lossy().to_string(),
                output_dir: Some(output_dir.to_string_lossy().to_string()),
                target_format: target_ext.clone(),
                crf: None,
                resolution: None,
                hardware_accel: false,
                selected_encoder: None,
                strip_metadata: false,
                audio_bitrate: None,
            };

            let start = std::time::Instant::now();
            let result = convert_single_file(handle.clone(), req).await;
            let duration = start.elapsed();

            let success = match &result {
                Ok(r) => r.success,
                Err(_) => false,
            };

            let error_msg = match &result {
                Ok(r) => r.error.clone(),
                Err(e) => Some(e.clone()),
            };

            results.push(MatrixResult {
                input: input_ext.clone(),
                output: target_ext.clone(),
                success,
                duration_ms: duration.as_millis() as u64,
                error: error_msg,
                engine: target.sidecar_engine,
            });
        }
    }

    generate_reports(results);
}

#[derive(Serialize)]
struct MatrixResult {
    input: String,
    output: String,
    success: bool,
    duration_ms: u64,
    error: Option<String>,
    engine: String,
}

fn create_minimal_valid_file(path: &Path, ext: &str) {
    match ext {
        "png" => fs::write(path, &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13, 0x49, 0x48, 0x44, 0x52, 0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0, 0x90, 0x77, 0x53, 0xDE, 0, 0, 0, 0, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]).unwrap(),
        "jpg" | "jpeg" => fs::write(path, &[0xFF, 0xD8, 0xFF, 0xDB, 0, 0, 0xFF, 0xD9]).unwrap(),
        "txt" | "md" | "html" | "csv" | "json" | "yaml" | "toml" | "log" | "bib" | "ics" => fs::write(path, "test data").unwrap(),
        "xml" => fs::write(path, "<root></root>").unwrap(),
        "zip" => {
            // Minimal empty zip
            fs::write(path, &[0x50, 0x4B, 0x05, 0x06, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]).unwrap();
        },
        _ => {
            // For media files, we might need a real tool to generate if they are missing
            // But for now just create a 1-byte file to see if the engine rejects it correctly
            fs::write(path, &[0]).unwrap();
        }
    }
}

fn generate_reports(results: Vec<MatrixResult>) {
    let mut md = String::from("# File2File Conversion Matrix Report\n\n");
    md.push_str("| Input | Output | Status | Engine | Duration | Error |\n");
    md.push_str("|-------|--------|--------|--------|----------|-------|\n");

    for r in results.iter() {
        let status = if r.success { "✅ OK" } else { "❌ FAIL" };
        md.push_str(&format!(
            "| .{} | .{} | {} | {} | {}ms | {} |\n",
            r.input, r.output, status, r.engine, r.duration_ms, r.error.as_ref().unwrap_or(&String::new()).replace('|', "\\|")
        ));
    }

    fs::write("../matrix_report.md", md).unwrap();

    let json = serde_json::to_string_pretty(&results).unwrap();
    fs::write("../matrix_report.json", json).unwrap();

    println!("Matrix reports generated: matrix_report.md and matrix_report.json");
}
