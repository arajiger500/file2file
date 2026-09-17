use file2file_lib::converter::{convert_single_file, ConversionRequest};
use std::fs;
use std::path::Path;
use futures::future::join_all;

#[tokio::test]
async fn test_large_batch_parallel() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let batch_dir = Path::new("../stress_batch_files");
    if batch_dir.exists() {
        fs::remove_dir_all(batch_dir).unwrap();
    }
    fs::create_dir_all(batch_dir).unwrap();

    let output_dir = Path::new("../stress_batch_output");
    if output_dir.exists() {
        fs::remove_dir_all(output_dir).unwrap();
    }
    fs::create_dir_all(output_dir).unwrap();

    let batch_size = 50;
    let mut futures = Vec::new();

    for i in 0..batch_size {
        let input_path = batch_dir.join(format!("file_{}.txt", i));
        fs::write(&input_path, format!("stress test data {}", i)).unwrap();

        let req = ConversionRequest {
            input_path: input_path.to_string_lossy().to_string(),
            output_dir: Some(output_dir.to_string_lossy().to_string()),
            target_format: "html".to_string(),
            crf: None,
            resolution: None,
            hardware_accel: false,
            selected_encoder: None,
            strip_metadata: false,
            audio_bitrate: None,
        };

        futures.push(convert_single_file(handle.clone(), req));
    }

    println!("Starting parallel batch conversion of {} files...", batch_size);
    let results = join_all(futures).await;

    let mut success_count = 0;
    for res in results {
        match res {
            Ok(r) if r.success => success_count += 1,
            Ok(r) => println!("Conversion failed: {:?}", r.error),
            Err(e) => println!("Error: {}", e),
        }
    }

    println!("Batch complete: {}/{} successful", success_count, batch_size);
    assert_eq!(success_count, batch_size, "Not all files converted successfully");
}

#[tokio::test]
#[ignore] // Run manually for long stress tests
async fn test_massive_file_handle() {
    // This would test a multi-GB file if available
}
