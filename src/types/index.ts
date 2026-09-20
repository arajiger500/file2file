export type FileCategory = "video" | "audio" | "image" | "document" | "vector" | "data" | "archive" | "unknown";

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
    absolute_path?: string | null;
}

export interface CategoryStatus {
    category: string;
    ready: boolean;
    engine: string;
    message?: string | null;
}

export interface SidecarHealthReport {
    binaries: BinaryStatus[];
    categories: CategoryStatus[];
    all_ready: boolean;
    ffmpeg: BinaryStatus;
    ffprobe: BinaryStatus;
    pandoc: BinaryStatus;
    magick: BinaryStatus;
    imagemagick: BinaryStatus;
    pdftotext: BinaryStatus;
    pdftohtml: BinaryStatus;
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
    collisionPolicy: CollisionPolicy;
    maxParallelJobs: number;
}

export type JobStatus = "pending" | "converting" | "completed" | "error" | "cancelled";

export interface FileItem {
    id: string;
    path: string;
    name: string;
    size: number;
    extension: string;
    category: FileCategory;
    status: JobStatus;
    targetFormat?: string;
    rawFile?: File;
    result?: ConversionResult;
}

export type CollisionPolicy = "overwrite" | "autorename" | "skip" | "ask";

export interface ConversionRequest {
    job_id?: string;
    input_path: string;
    output_dir?: string;
    target_format: string;
    crf?: number;
    resolution?: string;
    hardware_accel: boolean;
    selected_encoder?: string;
    strip_metadata: boolean;
    audio_bitrate?: string;
    collision_policy?: CollisionPolicy;
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
    warnings?: string[];
}

export interface FileProbeResult {
    has_video: boolean;
    has_audio: boolean;
    duration: number;
    width: number;
    height: number;
    format_name: string;
}

export interface ValidationResult {
    is_valid: boolean;
    warnings: string[];
    error: string | null;
    file_info: FileProbeResult | null;
}
