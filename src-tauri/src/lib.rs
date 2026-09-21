use std::path::Path;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
pub mod archive;
pub mod inputs;
pub mod jobs;
pub mod output;
pub mod process;
pub use jobs::AppState;

pub mod converter;
pub mod errors;
pub mod formats;
pub mod hardware;
pub mod registry;
pub mod sidecar;

mod commands {
    use super::*;
    use converter::ValidationResult;

    #[tauri::command]
    pub async fn detect_hardware(app: AppHandle) -> HardwareInfo {
        detect_hardware_capabilities(Some(app)).await
    }

    #[tauri::command]
    pub fn get_compatible_targets(input_ext: String) -> Vec<FormatOption> {
        get_compatible_formats(&input_ext)
    }

    #[tauri::command]
    pub fn get_smart_suggestions(input_ext: String) -> Vec<FormatOption> {
        formats::get_smart_recommendations(&input_ext)
    }

    #[tauri::command]
    pub fn get_presets() -> Vec<QuickPreset> {
        get_quick_presets()
    }

    #[tauri::command]
    pub async fn check_sidecars(app: AppHandle) -> SidecarHealthReport {
        check_sidecar_health(&app).await
    }

    #[tauri::command]
    pub async fn validate_job(
        app: AppHandle,
        input_path: String,
        target_format: String,
    ) -> ValidationResult {
        converter::validate_conversion(&app, &input_path, &target_format).await
    }

    #[tauri::command]
    pub async fn start_conversion(
        app: AppHandle,
        request: ConversionRequest,
    ) -> Result<ConversionResult, String> {
        convert_single_file(app, request).await
    }

    #[tauri::command]
    pub async fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
        state.cancel(&job_id)
    }

    #[tauri::command]
    pub fn show_in_folder(app: AppHandle, path: String) -> Result<(), String> {
        let path = Path::new(&path)
            .canonicalize()
            .map_err(|e| format!("Output path error: {e}"))?;
        app.opener()
            .reveal_item_in_dir(path)
            .map_err(|e| e.to_string())
    }
}

use converter::{convert_single_file, ConversionRequest, ConversionResult};
use formats::{get_compatible_formats, get_quick_presets, FormatOption, QuickPreset};
use hardware::{detect_hardware_capabilities, HardwareInfo};
use sidecar::{check_sidecar_health, SidecarHealthReport};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let context = tauri::generate_context!();

    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_hardware,
            commands::get_compatible_targets,
            commands::get_smart_suggestions,
            commands::get_presets,
            commands::check_sidecars,
            commands::validate_job,
            commands::start_conversion,
            commands::cancel_job,
            commands::show_in_folder,
            inputs::scan_inputs,
        ])
        .build(context)
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if app.state::<AppState>().shutdown() {
                    api.prevent_exit();
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        while !app.state::<AppState>().is_empty() {
                            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                        }
                        app.exit(0);
                    });
                }
            }
        });
}
