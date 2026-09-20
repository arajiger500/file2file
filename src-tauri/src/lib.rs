use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::{Mutex, Semaphore};

pub struct AppState {
    pub active_jobs: Mutex<HashMap<String, tauri_plugin_shell::process::CommandChild>>,
    pub conversion_semaphore: Arc<Semaphore>,
}

impl Default for AppState {
    fn default() -> Self {
        let cpu_cores = num_cpus::get();
        let max_concurrency = std::cmp::min(cpu_cores, 4).max(1);
        Self {
            active_jobs: Mutex::new(HashMap::new()),
            conversion_semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }
}

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
    pub async fn cancel_job(
        state: State<'_, AppState>,
        job_id: String,
    ) -> Result<(), String> {
        let mut jobs = state.active_jobs.lock().await;
        if let Some(child) = jobs.remove(&job_id) {
            child.kill().map_err(|e| format!("Failed to kill job: {}", e))?;
            return Ok(());
        }
        Err("Job not found or already completed".to_string())
    }

    #[tauri::command]
    pub fn copy_file(src: String, dest: String) -> Result<(), String> {
        let src_path = Path::new(&src);
        if !src_path.exists() {
            return Err("Source file does not exist".to_string());
        }
        let src_path = src_path.canonicalize().map_err(|e| format!("Source path error: {}", e))?;
        if !src_path.is_file() {
            return Err("Source is not a regular file".to_string());
        }

        if dest.contains("..") || dest.contains("\0") {
            return Err("Path traversal or null bytes not allowed".to_string());
        }
        let dest_path = Path::new(&dest);
        if dest_path.exists() {
            return Err("Destination file already exists".to_string());
        }
        if let Some(parent) = dest_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create destination directory: {}", e))?;
            }
        }
        std::fs::copy(&src_path, dest_path).map_err(|e| format!("Failed to copy file: {}", e))?;
        Ok(())
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
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
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
            commands::copy_file,
        ])
        .run(context)
        .expect("error while running tauri application");
}
