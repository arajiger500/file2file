use file2file_lib::converter::{convert_single_file, ConversionRequest};
use file2file_lib::sidecar::get_binary_command;
use std::fs;
use std::time::Instant;
use tempfile::tempdir;

#[tokio::test]
async fn benchmark_large_pdf_conversion() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let dir = tempdir().unwrap();
    let png_path = dir.path().join("image.png");
    let input_path = dir.path().join("large_test.pdf");

    // 1. Create a source image
    let magick_cmd = get_binary_command(handle, "magick").await.unwrap();
    let _ = magick_cmd.args([
        "-size", "100x100", "xc:blue",
        png_path.to_str().unwrap()
    ]).output().await.unwrap();

    // 2. Create a 5-page PDF for test
    let mut args = Vec::new();
    for _ in 0..5 {
        args.push(png_path.to_str().unwrap().to_string());
    }
    args.push(input_path.to_str().unwrap().to_string());

    let magick_cmd2 = get_binary_command(handle, "magick").await.unwrap();
    let output = magick_cmd2
        .args(args)
        .output()
        .await
        .unwrap();

    assert!(output.status.success(), "Failed to create test PDF: {}", String::from_utf8_lossy(&output.stderr));

    // 3. Measure conversion time
    let mut req = ConversionRequest::new(input_path.to_str().unwrap(), "docx");
    req.output_dir = Some(dir.path().to_str().unwrap().to_string());

    let start = Instant::now();
    let result = convert_single_file(handle.clone(), req).await.unwrap();
    let duration = start.elapsed();

    assert!(result.success, "Conversion failed: {:?}", result.error);
    println!("Converted PDF with images to DOCX in {}ms", duration.as_millis());

    let metadata = fs::metadata(&result.output_path).unwrap();
    assert!(metadata.len() > 0);
}
