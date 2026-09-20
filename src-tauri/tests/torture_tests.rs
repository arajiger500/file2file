use file2file_lib::converter::{convert_single_file, ConversionRequest};
use file2file_lib::sidecar::get_binary_command;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[tokio::test]
async fn test_transparency_preservation() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let dir = tempdir().unwrap();
    let input_path = dir.path().join("transparent.png");

    // Create a transparent PNG: a 100x100 image with a blue square in the middle and transparency around it
    let magick_cmd = get_binary_command(handle, "magick").await.unwrap();
    let output = magick_cmd
        .args([
            "-size", "100x100",
            "xc:none",
            "-fill", "blue",
            "-draw", "rectangle 25,25 75,75",
            input_path.to_str().unwrap(),
        ])
        .output()
        .await
        .unwrap();
    assert!(output.status.success(), "Failed to create transparent PNG: {}", String::from_utf8_lossy(&output.stderr));

    let mut req = ConversionRequest::new(input_path.to_str().unwrap(), "jpg");
    req.output_dir = Some(dir.path().to_str().unwrap().to_string());

    let result = convert_single_file(handle.clone(), req).await.unwrap();
    assert!(result.success, "Conversion failed: {:?}", result.error);

    let output_path = Path::new(&result.output_path);
    assert!(output_path.exists(), "Output JPG was not created");

    // Verify it's not a black block. Flattening against white background gives high mean.
    let magick_probe = get_binary_command(handle, "magick").await.unwrap();
    let probe_output = magick_probe
        .args([
            "identify",
            "-format", "%[mean]",
            output_path.to_str().unwrap(),
        ])
        .output()
        .await
        .unwrap();

    let mean_str = String::from_utf8_lossy(&probe_output.stdout).trim().to_string();
    let mean: f64 = mean_str.parse().unwrap_or(0.0);
    assert!(mean > 1000.0, "Output JPG appears to be a black block (mean: {})", mean);
}

#[tokio::test]
async fn test_ffmpeg_metadata_preservation() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let dir = tempdir().unwrap();
    let input_path = dir.path().join("metadata_input.mp4");

    // Create a dummy video with specific metadata
    let ffmpeg_cmd = get_binary_command(handle, "ffmpeg").await.unwrap();
    let output = ffmpeg_cmd
        .args([
            "-f", "lavfi",
            "-i", "color=c=blue:s=100x100:d=1",
            "-metadata", "title=TortureTestTitle",
            "-metadata", "artist=File2FileTester",
            input_path.to_str().unwrap(),
        ])
        .output()
        .await
        .unwrap();
    assert!(output.status.success(), "Failed to create metadata video: {}", String::from_utf8_lossy(&output.stderr));

    let mut req = ConversionRequest::new(input_path.to_str().unwrap(), "mkv");
    req.output_dir = Some(dir.path().to_str().unwrap().to_string());

    let result = convert_single_file(handle.clone(), req).await.unwrap();
    assert!(result.success, "Conversion failed: {:?}", result.error);

    let output_path = Path::new(&result.output_path);
    assert!(output_path.exists(), "Output MKV was not created");

    // Verify metadata survival with ffprobe
    let ffprobe_cmd = get_binary_command(handle, "ffprobe").await.unwrap();
    let probe_output = ffprobe_cmd
        .args([
            "-v", "quiet",
            "-show_entries", "format_tags=title,artist",
            "-of", "default=noprint_wrappers=1:nokey=1",
            output_path.to_str().unwrap(),
        ])
        .output()
        .await
        .unwrap();

    let probe_str = String::from_utf8_lossy(&probe_output.stdout).to_string();
    assert!(probe_str.contains("TortureTestTitle"), "Title metadata lost. Probe output: {}", probe_str);
    assert!(probe_str.contains("File2FileTester"), "Artist metadata lost. Probe output: {}", probe_str);
}

#[tokio::test]
async fn test_pandoc_math_preservation() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let dir = tempdir().unwrap();
    let input_path = dir.path().join("math.md");

    // Create a markdown file with LaTeX math
    fs::write(&input_path, "# Math Test\n\nHere is some math: $x^2 + y^2 = z^2$\n\nAnd a block:\n$$\nE = mc^2\n$$\n").unwrap();

    let mut req = ConversionRequest::new(input_path.to_str().unwrap(), "html");
    req.output_dir = Some(dir.path().to_str().unwrap().to_string());

    let result = convert_single_file(handle.clone(), req).await.unwrap();
    assert!(result.success, "Conversion failed: {:?}", result.error);

    let output_path = Path::new(&result.output_path);
    let html_content = fs::read_to_string(output_path).unwrap();

    assert!(html_content.contains("math inline") || html_content.contains("math display") || html_content.contains("x^2"),
        "Math content seems lost in HTML output");
}
