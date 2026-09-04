use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderProfile {
    pub id: String,
    pub name: String,
    pub codec: String,
    pub is_hardware: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub gpu_vendor: Option<String>,
    pub hardware_acceleration_supported: bool,
    pub recommended_encoder: String,
    pub cpu_cores: usize,
    pub available_encoders: Vec<EncoderProfile>,
    pub detected_gpus: Vec<String>,
}

use tauri_plugin_shell::ShellExt;

pub async fn detect_hardware_capabilities(app: Option<tauri::AppHandle>) -> HardwareInfo {
    let cpu_cores = num_cpus::get();
    let mut detected_gpus = Vec::new();
    let mut available_encoders = Vec::new();
    let mut gpu_vendor = None;

    // Detect platform-specific GPU indicators
    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/dev/dri").exists() {
            detected_gpus.push("VA-API Compatible Display Adapter".to_string());
        }
        if std::path::Path::new("/proc/driver/nvidia/version").exists() {
            gpu_vendor = Some("NVIDIA".to_string());
            detected_gpus.push("NVIDIA GPU (NVENC Supported)".to_string());
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Simple heuristic / fallback for Windows
        detected_gpus.push("Windows Direct3D / MediaFoundation Adapter".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        gpu_vendor = Some("Apple Silicon / Metal".to_string());
        detected_gpus.push("Apple VideoToolbox Framework".to_string());
    }

    // Try querying ffmpeg encoders if available on PATH or sidecar
    let ffmpeg_output = if let Some(app) = app {
        if let Ok(sidecar) = app.shell().sidecar("ffmpeg") {
            sidecar.args(["-encoders"]).output().await
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        Command::new("ffmpeg")
            .arg("-encoders")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .unwrap_or_default()
    };

    let has_nvenc = ffmpeg_output.contains("nvenc") || gpu_vendor.as_deref() == Some("NVIDIA");
    let has_qsv = ffmpeg_output.contains("qsv");
    let has_amf = ffmpeg_output.contains("amf") || ffmpeg_output.contains("vaapi");
    let has_videotoolbox = ffmpeg_output.contains("videotoolbox") || cfg!(target_os = "macos");

    if has_nvenc {
        gpu_vendor = Some("NVIDIA".to_string());
        available_encoders.push(EncoderProfile {
            id: "h264_nvenc".to_string(),
            name: "NVIDIA NVENC H.264".to_string(),
            codec: "h264".to_string(),
            is_hardware: true,
            description: "High-speed hardware encoding via NVIDIA GPU".to_string(),
        });
        available_encoders.push(EncoderProfile {
            id: "hevc_nvenc".to_string(),
            name: "NVIDIA NVENC HEVC/H.265".to_string(),
            codec: "hevc".to_string(),
            is_hardware: true,
            description: "Next-gen compression using NVIDIA GPU acceleration".to_string(),
        });
    }

    if has_qsv {
        if gpu_vendor.is_none() {
            gpu_vendor = Some("Intel".to_string());
        }
        available_encoders.push(EncoderProfile {
            id: "h264_qsv".to_string(),
            name: "Intel QuickSync H.264".to_string(),
            codec: "h264".to_string(),
            is_hardware: true,
            description: "Hardware accelerated encoding via Intel QuickSync".to_string(),
        });
    }

    if has_amf {
        if gpu_vendor.is_none() {
            gpu_vendor = Some("AMD / VA-API".to_string());
        }
        available_encoders.push(EncoderProfile {
            id: "h264_vaapi".to_string(),
            name: "VA-API / AMF H.264".to_string(),
            codec: "h264".to_string(),
            is_hardware: true,
            description: "Hardware accelerated encoding via AMD/VA-API pipeline".to_string(),
        });
    }

    if has_videotoolbox {
        available_encoders.push(EncoderProfile {
            id: "h264_videotoolbox".to_string(),
            name: "Apple VideoToolbox H.264".to_string(),
            codec: "h264".to_string(),
            is_hardware: true,
            description: "Native hardware acceleration for Apple Silicon / Intel Mac".to_string(),
        });
    }

    // Always provide multi-threaded software encoding fallback
    available_encoders.push(EncoderProfile {
        id: "libx264".to_string(),
        name: format!("Multi-Threaded CPU (libx264 - {} cores)", cpu_cores),
        codec: "h264".to_string(),
        is_hardware: false,
        description: "Universal CPU software encoder. High compatibility and quality.".to_string(),
    });

    available_encoders.push(EncoderProfile {
        id: "libx265".to_string(),
        name: format!("Multi-Threaded CPU (libx265 - {} cores)", cpu_cores),
        codec: "hevc".to_string(),
        is_hardware: false,
        description: "High-efficiency CPU software encoder for modern devices.".to_string(),
    });

    let hw_supported = available_encoders.iter().any(|e| e.is_hardware);
    let recommended_encoder = available_encoders
        .iter()
        .find(|e| e.is_hardware)
        .map(|e| e.id.clone())
        .unwrap_or_else(|| "libx264".to_string());

    HardwareInfo {
        gpu_vendor,
        hardware_acceleration_supported: hw_supported,
        recommended_encoder,
        cpu_cores,
        available_encoders,
        detected_gpus,
    }
}
