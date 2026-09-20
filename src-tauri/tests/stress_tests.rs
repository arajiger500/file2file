use file2file_lib::converter::{convert_single_file, ConversionRequest};
use futures::future::join_all;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_large_batch_parallel() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let batch_dir = tempdir().unwrap();
    let output_dir = tempdir().unwrap();

    let batch_size = 20;
    let mut futures = Vec::new();

    for i in 0..batch_size {
        let input_path = batch_dir.path().join(format!("file_{}.txt", i));
        fs::write(&input_path, format!("stress test data content {}", i)).unwrap();

        let mut req = ConversionRequest::new(input_path.to_str().unwrap(), "html");
        req.output_dir = Some(output_dir.path().to_str().unwrap().to_string());

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
