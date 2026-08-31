use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileCategory {
    Video,
    Audio,
    Image,
    Document,
    Vector,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatOption {
    pub extension: String,
    pub name: String,
    pub category: FileCategory,
    pub subcategory: String,
    pub description: String,
    pub comparison_note: Option<String>,
    pub is_lossless: bool,
    pub is_recommended: bool,
    pub recommended_for: Vec<String>,
    pub sidecar_engine: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickPreset {
    pub id: String,
    pub title: String,
    pub from_category: FileCategory,
    pub to_format: String,
    pub description: String,
    pub badge: String,
    pub icon: String,
    pub target_name: String,
}

pub fn get_category_for_extension(ext: &str) -> FileCategory {
    match ext.to_lowercase().as_str() {
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "flv" | "wmv" | "m4v" | "ts" | "3gp" | "ogv"
        | "vob" | "gif" => FileCategory::Video,
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "opus" | "wma" | "aiff" => {
            FileCategory::Audio
        }
        "png" | "jpg" | "jpeg" | "webp" | "avif" | "bmp" | "tiff" | "ico" | "heic" | "tga"
        | "psd" => FileCategory::Image,
        "pdf" | "docx" | "doc" | "md" | "html" | "txt" | "epub" | "rtf" | "odt" => {
            FileCategory::Document
        }
        "svg" | "eps" | "ai" => FileCategory::Vector,
        _ => FileCategory::Unknown,
    }
}

pub fn get_compatible_formats(input_ext: &str) -> Vec<FormatOption> {
    let category = get_category_for_extension(input_ext);
    let mut catalog = Vec::new();

    match category {
        FileCategory::Document => {
            let targets = ["pdf", "docx", "md", "html", "txt", "rtf", "epub", "odt"];
            for t in targets {
                if t == input_ext.to_lowercase() {
                    continue;
                }
                catalog.push(FormatOption {
                    extension: t.to_string(),
                    name: format!("{} Document", t.to_uppercase()),
                    category: FileCategory::Document,
                    subcategory: "Universal".to_string(),
                    description: format!("Structural {} conversion", t.to_uppercase()),
                    comparison_note: None,
                    is_lossless: true,
                    is_recommended: true,
                    recommended_for: vec!["Editing".to_string()],
                    sidecar_engine: "Pro-Core".to_string(),
                    pros: vec!["High Fidelity".to_string()],
                    cons: vec![],
                });
            }
        }
        FileCategory::Image | FileCategory::Vector => {
            let targets = [
                "webp", "avif", "png", "jpg", "ico", "bmp", "tiff", "pdf", "tga",
            ];
            for t in targets {
                if t == input_ext.to_lowercase() {
                    continue;
                }
                catalog.push(FormatOption {
                    extension: t.to_string(),
                    name: format!("{} Graphics", t.to_uppercase()),
                    category: if t == "pdf" {
                        FileCategory::Document
                    } else {
                        FileCategory::Image
                    },
                    subcategory: "Studio".to_string(),
                    description: format!("Pixel-perfect {} encoding", t.to_uppercase()),
                    comparison_note: None,
                    is_lossless: ["png", "bmp", "tiff"].contains(&t),
                    is_recommended: true,
                    recommended_for: vec!["Web".to_string(), "Creative".to_string()],
                    sidecar_engine: "ImageMagick".to_string(),
                    pros: vec!["Clean".to_string()],
                    cons: vec![],
                });
            }
        }
        FileCategory::Video => {
            let targets = [
                "mp4", "webm", "mkv", "mov", "avi", "gif", "mp3", "wav", "flac",
            ];
            for t in targets {
                if t == input_ext.to_lowercase() {
                    continue;
                }
                catalog.push(FormatOption {
                    extension: t.to_string(),
                    name: format!("{} Media", t.to_uppercase()),
                    category: if ["mp3", "wav", "flac"].contains(&t) {
                        FileCategory::Audio
                    } else {
                        FileCategory::Video
                    },
                    subcategory: "Broadcast".to_string(),
                    description: format!("Native FFmpeg {} pipeline", t.to_uppercase()),
                    comparison_note: None,
                    is_lossless: false,
                    is_recommended: true,
                    recommended_for: vec!["All Devices".to_string()],
                    sidecar_engine: "FFmpeg".to_string(),
                    pros: vec!["Standard".to_string()],
                    cons: vec![],
                });
            }
        }
        FileCategory::Audio => {
            let targets = ["mp3", "wav", "flac", "aac", "ogg", "m4a", "opus"];
            for t in targets {
                if t == input_ext.to_lowercase() {
                    continue;
                }
                catalog.push(FormatOption {
                    extension: t.to_string(),
                    name: format!("{} Audio", t.to_uppercase()),
                    category: FileCategory::Audio,
                    subcategory: "HiFi".to_string(),
                    description: format!("Accurate {} audio master", t.to_uppercase()),
                    comparison_note: None,
                    is_lossless: ["flac", "wav"].contains(&t),
                    is_recommended: true,
                    recommended_for: vec!["Music".to_string()],
                    sidecar_engine: "FFmpeg".to_string(),
                    pros: vec!["Zero Jitter".to_string()],
                    cons: vec![],
                });
            }
        }
        _ => {}
    }

    catalog
}

pub fn get_quick_presets() -> Vec<QuickPreset> {
    vec![
        QuickPreset {
            id: "p1".to_string(),
            title: "PDF to Editable Word".to_string(),
            target_name: "Word".to_string(),
            from_category: FileCategory::Document,
            to_format: "docx".to_string(),
            description: "Real text extraction".to_string(),
            badge: "Pro".to_string(),
            icon: "file-text".to_string(),
        },
        QuickPreset {
            id: "p2".to_string(),
            title: "Image to WebP".to_string(),
            target_name: "WebP".to_string(),
            from_category: FileCategory::Image,
            to_format: "webp".to_string(),
            description: "Next-gen compression".to_string(),
            badge: "Fast".to_string(),
            icon: "image".to_string(),
        },
        QuickPreset {
            id: "p3".to_string(),
            title: "Video to MP4".to_string(),
            target_name: "MP4".to_string(),
            from_category: FileCategory::Video,
            to_format: "mp4".to_string(),
            description: "Universal playback".to_string(),
            badge: "Media".to_string(),
            icon: "video".to_string(),
        },
        QuickPreset {
            id: "p4".to_string(),
            title: "Video to MP3".to_string(),
            target_name: "MP3".to_string(),
            from_category: FileCategory::Video,
            to_format: "mp3".to_string(),
            description: "Extract audio track".to_string(),
            badge: "Music".to_string(),
            icon: "music".to_string(),
        },
    ]
}
