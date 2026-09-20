use file2file_lib::converter::{convert_single_file, ConversionRequest};
use std::fs::{self, File};
use std::io::Read;
use tempfile::tempdir;

#[tokio::test]
async fn test_html_to_docx_table_preservation() {
    let app = tauri::test::mock_builder()
        .plugin(tauri_plugin_shell::init())
        .build(tauri::generate_context!())
        .unwrap();
    let handle = app.handle();

    let dir = tempdir().unwrap();
    let html_path = dir.path().join("source.html");

    // Create an HTML with a table
    let table_html = r#"<!DOCTYPE html>
<html>
<body>
  <table>
    <thead><tr><th>Col 1</th><th>Col 2</th></tr></thead>
    <tbody><tr><td>Cell 1-1</td><td>Cell 1-2</td></tr></tbody>
  </table>
</body>
</html>"#;
    fs::write(&html_path, table_html).unwrap();

    let mut req_docx = ConversionRequest::new(html_path.to_str().unwrap(), "docx");
    req_docx.output_dir = Some(dir.path().to_str().unwrap().to_string());

    let result = convert_single_file(handle.clone(), req_docx).await.unwrap();
    assert!(result.success, "HTML to DOCX conversion failed: {:?}", result.error);

    // Verify table in DOCX by checking XML content
    let file = File::open(&result.output_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut document_xml = String::new();
    archive.by_name("word/document.xml").unwrap().read_to_string(&mut document_xml).unwrap();

    assert!(document_xml.contains("<w:tbl>"), "No table found in DOCX");
    assert!(document_xml.contains("Cell 1-1"), "Table content lost in DOCX");
}
