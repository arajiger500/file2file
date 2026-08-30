pub mod converter;
pub mod formats;
pub mod hardware;
pub mod sidecar;

mod commands {
    use super::*;
    use tauri::AppHandle;

    #[tauri::command]
    pub fn detect_hardware() -> HardwareInfo {
        detect_hardware_capabilities()
    }

    #[tauri::command]
    pub fn get_compatible_targets(input_ext: String) -> Vec<FormatOption> {
        get_compatible_formats(&input_ext)
    }

    #[tauri::command]
    pub fn get_presets() -> Vec<QuickPreset> {
        get_quick_presets()
    }

    #[tauri::command]
    pub fn check_sidecars() -> SidecarHealthReport {
        check_sidecar_health()
    }

    #[tauri::command]
    pub async fn start_conversion(app: AppHandle, request: ConversionRequest) -> Result<ConversionResult, String> {
        convert_single_file(app, request).await
    }

    #[tauri::command]
    pub fn copy_file(src: String, dest: String) -> Result<(), String> {
        std::fs::copy(&src, &dest)
            .map(|_| ())
            .map_err(|e| format!("Failed to copy file: {}", e))
    }

    #[tauri::command]
    pub fn write_base64_file(path: String, b64: String) -> Result<(), String> {
        match base64::decode(&b64) {
            Ok(bytes) => std::fs::write(&path, &bytes).map_err(|e| format!("Failed to write file: {}", e)),
            Err(e) => Err(format!("Base64 decode error: {}", e)),
        }
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
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_hardware,
            commands::get_compatible_targets,
            commands::get_presets,
            commands::check_sidecars,
            commands::start_conversion,
            commands::copy_file,
            commands::write_base64_file,
        ])
        .run(context)
        .expect("error while running tauri application");
}
