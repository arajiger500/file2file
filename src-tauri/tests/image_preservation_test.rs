use file2file_lib::converter::{convert_single_file, ConversionRequest};
use file2file_lib::sidecar::get_binary_command;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[tokio::test]
async fn test_pdf_to_docx_image_preservation() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let dir = tempdir().unwrap();
    let input_path = dir.path().join("test_with_image.pdf");
    let png_path = dir.path().join("source.png");

    // Create a real PNG first
    let magick_cmd = get_binary_command(handle, "magick").await.unwrap();
    let _ = magick_cmd.args([
        "-size", "100x100", "xc:blue",
        png_path.to_str().unwrap()
    ]).output().await.unwrap();

    // Convert PNG to PDF to ensure it contains a raster image
    let magick_cmd2 = get_binary_command(handle, "magick").await.unwrap();
    let output = magick_cmd2
        .args([
            png_path.to_str().unwrap(),
            input_path.to_str().unwrap()
        ])
        .output()
        .await
        .unwrap();

    assert!(output.status.success(), "magick failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(input_path.exists(), "Test PDF was not created");

    let req = ConversionRequest {
        input_path: input_path.to_str().unwrap().to_string(),
        output_dir: Some(dir.path().to_str().unwrap().to_string()),
        target_format: "docx".to_string(),
        crf: None,
        resolution: None,
        hardware_accel: false,
        selected_encoder: None,
        strip_metadata: false,
        audio_bitrate: None,
    };

    let result = convert_single_file(handle.clone(), req).await.unwrap();
    assert!(result.success, "Conversion failed: {:?}", result.error);

    let output_path = Path::new(&result.output_path);
    assert!(output_path.exists(), "Output DOCX was not created");

    // Verify image preservation by checking ZIP content
    let file = File::open(output_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let mut image_found = false;
    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        println!("DOCX Entry: {}", file.name());
        if file.name().starts_with("word/media/") {
            image_found = true;
            break;
        }
    }
    assert!(image_found, "No images found in the generated DOCX");
}
