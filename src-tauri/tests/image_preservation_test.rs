use file2file_lib::converter::{convert_single_file, ConversionRequest};
use file2file_lib::sidecar::get_binary_command;
use std::fs::File;
use std::path::Path;
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

    // Convert PNG to PDF using ImageMagick
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

    let mut req = ConversionRequest::new(input_path.to_str().unwrap(), "docx");
    req.output_dir = Some(dir.path().to_str().unwrap().to_string());

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
        if file.name().starts_with("word/media/") {
            image_found = true;
            break;
        }
    }
    assert!(image_found, "No images found in the generated DOCX");
}
