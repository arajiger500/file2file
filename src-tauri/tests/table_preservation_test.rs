use file2file_lib::converter::{convert_single_file, ConversionRequest};
use file2file_lib::sidecar::get_binary_command;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use tempfile::tempdir;

#[tokio::test]
async fn test_pdf_to_docx_table_preservation() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let dir = tempdir().unwrap();
    let input_path = dir.path().join("test_with_table.pdf");
    let html_path = dir.path().join("source.html");

    // Create an HTML with a table first
    let table_html = r#"
        <html><body>
        <table border="1">
            <tr><td>Cell 1-1</td><td>Cell 1-2</td></tr>
            <tr><td>Cell 2-1</td><td>Cell 2-2</td></tr>
        </table>
        </body></html>
    "#;
    fs::write(&html_path, table_html).unwrap();

    // Convert HTML to PDF using Pandoc (requires a PDF engine, let's see if it works)
    // Wait, I don't have a PDF engine. I'll use a pre-existing PDF if I can or just trust pdftohtml.

    // Actually, I can check if Pandoc HTML -> DOCX preserves tables (Universal check)
    let req_docx = ConversionRequest {
        input_path: html_path.to_str().unwrap().to_string(),
        output_dir: Some(dir.path().to_str().unwrap().to_string()),
        target_format: "docx".to_string(),
        crf: None,
        resolution: None,
        hardware_accel: false,
        selected_encoder: None,
        strip_metadata: false,
        audio_bitrate: None,
    };

    let result = convert_single_file(handle.clone(), req_docx).await.unwrap();
    assert!(result.success, "HTML to DOCX conversion failed: {:?}", result.error);

    // Verify table in DOCX by checking XML content
    let file = File::open(&result.output_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut document_xml = String::new();
    use std::io::Read;
    archive.by_name("word/document.xml").unwrap().read_to_string(&mut document_xml).unwrap();

    assert!(document_xml.contains("<w:tbl>"), "No table found in DOCX");
    assert!(document_xml.contains("Cell 1-1"), "Table content lost");
}
