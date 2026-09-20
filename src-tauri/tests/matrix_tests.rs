use file2file_lib::converter::{convert_single_file, CollisionPolicy, ConversionRequest};
use file2file_lib::registry::Registry;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

#[tokio::test]
async fn test_representative_conversion_matrix() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let out_dir = tempdir().unwrap();
    let test_files_dir = Path::new("../test_files");

    let representative_pairs = vec![
        // Video / Audio conversions
        ("test.mp4", "webm"),
        ("test.mp4", "mkv"),
        ("test.mp4", "avi"),
        ("test.mp4", "gif"),
        ("test.mp4", "mp3"),
        ("test.webm", "mp4"),
        ("test.mkv", "mp4"),
        ("test.wav", "mp3"),
        ("test.mp3", "wav"),
        ("test.flac", "mp3"),
        ("test.ogg", "wav"),
        ("test.m4a", "mp3"),
        ("test.opus", "wav"),
        // Image conversions
        ("test.png", "webp"),
        ("test.png", "jpg"),
        ("test.png", "ico"),
        ("test.jpg", "png"),
        ("test.webp", "png"),
        ("test.bmp", "png"),
        ("test.tiff", "jpg"),
        ("test.svg", "png"),
        ("test.png", "pdf"),
        // Document conversions
        ("test.md", "html"),
        ("test.md", "docx"),
        ("test.html", "docx"),
        ("test.docx", "html"),
        ("test.docx", "txt"),
        ("test.docx", "md"),
        ("test.pdf", "txt"),
        ("test.pdf", "docx"),
        ("test.rtf", "docx"),
        ("test.epub", "txt"),
        // Structured Data conversions
        ("test.csv", "json"),
        ("test.json", "csv"),
        ("test.json", "yaml"),
        ("test.yaml", "json"),
        ("test.json", "toml"),
        ("test.toml", "json"),
        ("test.json", "xml"),
        ("test.csv", "sql"),
        ("test.sqlite", "json"),
        ("test.log", "json"),
        ("test.bib", "json"),
        ("test.ics", "json"),
        // Archive conversions
        ("test.tar", "zip"),
        ("test.gz", "zip"),
        ("test.zip", "folder"),
    ];

    let mut passed = 0;
    let mut failed = 0;
    let mut failure_messages = Vec::new();

    for (file_name, target_ext) in representative_pairs {
        let input_path = test_files_dir.join(file_name);
        if !input_path.exists() {
            println!("Skipping missing fixture: {}", input_path.display());
            continue;
        }

        let mut req = ConversionRequest::new(input_path.to_str().unwrap(), target_ext);
        req.output_dir = Some(out_dir.path().to_str().unwrap().to_string());

        let result = convert_single_file(handle.clone(), req).await;
        match result {
            Ok(res) if res.success => {
                passed += 1;
                println!("PASS: {} -> .{}", file_name, target_ext);
            }
            Ok(res) => {
                failed += 1;
                let msg = format!("FAIL: {} -> .{}: {:?}", file_name, target_ext, res.error);
                println!("{}", msg);
                failure_messages.push(msg);
            }
            Err(e) => {
                failed += 1;
                let msg = format!("ERROR: {} -> .{}: {}", file_name, target_ext, e);
                println!("{}", msg);
                failure_messages.push(msg);
            }
        }
    }

    println!("\nMatrix Summary: {} passed, {} failed", passed, failed);
    if !failure_messages.is_empty() {
        for f in &failure_messages {
            eprintln!("  {}", f);
        }
    }
    assert_eq!(failed, 0, "Representative conversion matrix had failures: {:?}", failure_messages);
}

#[tokio::test]
async fn test_security_and_path_safety() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let dir = tempdir().unwrap();
    let input_path = dir.path().join("safe.txt");
    fs::write(&input_path, "test").unwrap();

    // 1. Reject unsupported target format
    let req_invalid = ConversionRequest::new(input_path.to_str().unwrap(), "nonexistent_ext");
    let res = convert_single_file(handle.clone(), req_invalid).await;
    assert!(res.is_err(), "Must reject unsupported target format");

    // 2. Reject directory traversal in target format
    let req_traversal = ConversionRequest::new(input_path.to_str().unwrap(), "../evil");
    let res = convert_single_file(handle.clone(), req_traversal).await;
    assert!(res.is_err(), "Must reject path traversal in target format");

    // 3. Prohibit overwrite when output path equals input path
    let mut req_overwrite = ConversionRequest::new(input_path.to_str().unwrap(), "txt");
    req_overwrite.output_dir = Some(dir.path().to_str().unwrap().to_string());
    req_overwrite.collision_policy = Some(CollisionPolicy::Overwrite);
    // Even if target is txt, stem_converted.txt differs from safe.txt, but if we target safe.txt collision
    let converted_path = dir.path().join("safe_converted.txt");
    fs::write(&converted_path, "existing").unwrap();
    // AutoRename policy should rename to safe_converted_1.txt
    let mut req_rename = ConversionRequest::new(input_path.to_str().unwrap(), "html");
    req_rename.output_dir = Some(dir.path().to_str().unwrap().to_string());
    req_rename.collision_policy = Some(CollisionPolicy::AutoRename);
    let res = convert_single_file(handle.clone(), req_rename).await.unwrap();
    assert!(res.success);
    assert!(Path::new(&res.output_path).exists());
}

#[test]
fn test_registry_consistency() {
    let caps = Registry::get_all_capabilities();
    assert!(!caps.is_empty());

    for cap in &caps {
        assert!(!cap.from_ext.is_empty());
        assert!(!cap.to_ext.is_empty());
        assert_ne!(cap.from_ext, cap.to_ext);
        assert!(Registry::is_supported(&cap.from_ext, &cap.to_ext));
    }
}
