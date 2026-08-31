export type FileCategory = "video" | "audio" | "image" | "document" | "vector" | "unknown";

export interface EncoderProfile {
    id: string;
    name: string;
    codec: string;
    is_hardware: boolean;
    description: string;
}

export interface HardwareInfo {
    gpu_vendor: string | null;
    hardware_acceleration_supported: boolean;
    recommended_encoder: string;
    cpu_cores: number;
    available_encoders: EncoderProfile[];
    detected_gpus: string[];
}

export interface BinaryStatus {
    name: string;
    available: boolean;
    version: string | null;
    path_or_sidecar: string;
}

export interface SidecarHealthReport {
    ffmpeg: BinaryStatus;
    pandoc: BinaryStatus;
    imagemagick: BinaryStatus;
    pdftotext: BinaryStatus;
    all_ready: boolean;
}

export interface FormatOption {
    extension: string;
    name: string;
    category: FileCategory;
    subcategory: string;
    description: string;
    comparison_note: string | null;
    is_lossless: boolean;
    is_recommended: boolean;
    recommended_for: string[];
    sidecar_engine: string;
    pros: string[];
    cons: string[];
}

export interface QuickPreset {
    id: string;
    title: string;
    from_category: FileCategory;
    to_format: string;
    description: string;
    badge: string;
    icon: string;
    target_name: string;
}

export interface AdvancedSettings {
    crf: number;
    resolution: string;
    hardwareAccel: boolean;
    selectedEncoder: string;
    stripMetadata: boolean;
    audioBitrate: string;
}

export interface FileItem {
    id: string;
    path: string;
    name: string;
    size: number;
    extension: string;
    category: FileCategory;
    status: "pending" | "converting" | "completed" | "error";
    targetFormat?: string;
    rawFile?: File;
    result?: ConversionResult;
}

export interface ConversionRequest {
    input_path: string;
    output_dir?: string;
    target_format: string;
    crf?: number;
    resolution?: string;
    hardware_accel: boolean;
    selected_encoder?: string;
    strip_metadata: boolean;
    audio_bitrate?: string;
    rawFile?: File;
}

export interface ConversionResult {
    job_id: string;
    input_path: string;
    output_path: string;
    success: boolean;
    original_size_bytes: number;
    converted_size_bytes: number;
    elapsed_ms: number;
    download_url?: string;
    error?: string;
}
