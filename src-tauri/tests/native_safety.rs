//! Real, generated fixtures; no engines, display server or network required.
use file2file_lib::{
    converter::{convert_single_file, CollisionPolicy, ConversionRequest},
    registry::Registry,
    AppState,
};
use std::{fs, path::Path};
use tauri::Manager;

fn app() -> tauri::App<tauri::test::MockRuntime> {
    tauri::test::mock_builder()
        .manage(AppState::default())
        .build(tauri::generate_context!())
        .unwrap()
}

#[test]
fn registry_routes_are_internally_consistent() {
    let capabilities = Registry::get_all_capabilities();
    assert!(!capabilities.is_empty());
    for capability in capabilities {
        assert!(!capability.from_ext.is_empty());
        assert!(!capability.to_ext.is_empty());
        assert_ne!(capability.from_ext, capability.to_ext);
        assert!(Registry::is_supported(
            &capability.from_ext,
            &capability.to_ext
        ));
    }
}

#[tokio::test]
async fn concurrent_jobs_publish_distinct_valid_outputs_and_clear_registry() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("café space $semicolon;.json");
    fs::write(&input, r#"[{"name":"東京","count":2}]"#).unwrap();
    let requests = (0..12).map(|_| {
        convert_single_file(
            app.handle().clone(),
            ConversionRequest::new(input.to_str().unwrap(), "yaml"),
        )
    });
    let results = futures::future::join_all(requests).await;
    let mut outputs = std::collections::HashSet::new();
    for result in results {
        let result = result.unwrap();
        assert!(result.success, "{:?}", result.error);
        let value: serde_json::Value =
            serde_yaml::from_str(&fs::read_to_string(&result.output_path).unwrap()).unwrap();
        assert_eq!(value[0]["name"], "東京");
        assert!(outputs.insert(result.output_path));
    }
    assert!(app.state::<AppState>().is_empty());
    assert!(!fs::read_dir(dir.path()).unwrap().any(|e| e
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".file2file-")));
}

#[tokio::test]
async fn malformed_inputs_and_unsafe_parameters_never_publish() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("bad.json");
    for content in ["", "{", "[1,2]", "[{\"ok\":1}, 3]"] {
        fs::write(&input, content).unwrap();
        let result = convert_single_file(
            app.handle().clone(),
            ConversionRequest::new(input.to_str().unwrap(), "csv"),
        )
        .await
        .unwrap();
        assert!(!result.success);
        assert!(!dir.path().join("bad_converted.csv").exists());
    }
    let mut req = ConversionRequest::new(input.to_str().unwrap(), "../escape");
    assert!(convert_single_file(app.handle().clone(), req.clone())
        .await
        .is_err());
    req.target_format = "yaml".into();
    req.selected_encoder = Some("-arbitrary-option".into());
    assert!(convert_single_file(app.handle().clone(), req)
        .await
        .is_err());
    assert!(app.state::<AppState>().is_empty());
}

#[tokio::test]
async fn cancelled_queued_job_never_runs_and_releases_registration() {
    let app = app();
    let state = app.state::<AppState>();
    let permits = state
        .conversion_semaphore
        .acquire_many(state.conversion_semaphore.available_permits() as u32)
        .await
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    fs::write(&input, "{}").unwrap();
    let mut req = ConversionRequest::new(input.to_str().unwrap(), "yaml");
    req.job_id = Some("queued".into());
    let conversion = convert_single_file(app.handle().clone(), req);
    let cancel = async {
        while state.subscribe("queued").is_err() {
            tokio::task::yield_now().await;
        }
        state.cancel("queued").unwrap();
    };
    let (result, _) = tokio::join!(conversion, cancel);
    assert!(result.unwrap_err().contains("cancelled"));
    drop(permits);
    assert!(state.is_empty());
    assert!(!dir.path().join("input_converted.yaml").exists());
}

#[tokio::test]
async fn xlsx_preserves_later_columns_and_large_integer_text() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("table.json");
    fs::write(&input, r#"[{"id":9007199254740993},{"later":"yes"}]"#).unwrap();
    let result = convert_single_file(
        app.handle().clone(),
        ConversionRequest::new(input.to_str().unwrap(), "xlsx"),
    )
    .await
    .unwrap();
    assert!(result.success, "{:?}", result.error);
    let result = convert_single_file(
        app.handle().clone(),
        ConversionRequest::new(result.output_path, "json"),
    )
    .await
    .unwrap();
    assert!(result.success, "{:?}", result.error);
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(result.output_path).unwrap()).unwrap();
    assert_eq!(value[0]["id"], "9007199254740993");
    assert_eq!(value[1]["later"], "yes");
}

#[tokio::test]
async fn skip_and_overwrite_are_explicit() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.json");
    let output = dir.path().join("input_converted.yaml");
    fs::write(&input, "{\"new\":true}").unwrap();
    fs::write(&output, "original").unwrap();
    let mut req = ConversionRequest::new(input.to_str().unwrap(), "yaml");
    req.collision_policy = Some(CollisionPolicy::Skip);
    let result = convert_single_file(app.handle().clone(), req.clone())
        .await
        .unwrap();
    assert!(result.success);
    assert!(!result.warnings.is_empty());
    assert_eq!(fs::read_to_string(&output).unwrap(), "original");
    req.collision_policy = Some(CollisionPolicy::Overwrite);
    let result = convert_single_file(app.handle().clone(), req)
        .await
        .unwrap();
    assert!(result.success);
    assert_eq!(
        Path::new(&result.output_path).canonicalize().unwrap(),
        output.canonicalize().unwrap()
    );
    assert!(fs::read_to_string(output).unwrap().contains("new: true"));
}

#[tokio::test]
async fn duplicate_csv_headers_are_errors() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("duplicate.csv");
    fs::write(&input, "same,same\n1,2\n").unwrap();
    let result = convert_single_file(
        app.handle().clone(),
        ConversionRequest::new(input.to_str().unwrap(), "json"),
    )
    .await
    .unwrap();
    assert!(!result.success);
    assert!(result.error.unwrap().contains("duplicate"));
}

#[tokio::test]
async fn spreadsheet_formula_cells_are_written_as_text() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("formula.json");
    fs::write(
        &input,
        r#"[{"cell":"=HYPERLINK(\"https://example.invalid\")"}]"#,
    )
    .unwrap();
    let result = convert_single_file(
        app.handle().clone(),
        ConversionRequest::new(input.to_str().unwrap(), "csv"),
    )
    .await
    .unwrap();
    assert!(result.success, "{:?}", result.error);
    assert!(fs::read_to_string(result.output_path)
        .unwrap()
        .contains("'=HYPERLINK"));
}
