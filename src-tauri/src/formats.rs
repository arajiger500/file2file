use crate::registry::Registry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileCategory {
    Video,
    Audio,
    Image,
    Document,
    Vector,
    Data,
    Archive,
    Unknown,
}

impl std::fmt::Display for FileCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FileCategory::Video => "Video",
            FileCategory::Audio => "Audio",
            FileCategory::Image => "Image",
            FileCategory::Document => "Document",
            FileCategory::Vector => "Vector",
            FileCategory::Data => "Data",
            FileCategory::Archive => "Archive",
            FileCategory::Unknown => "File",
        };
        write!(f, "{}", s)
    }
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
    let all = Registry::get_all_capabilities();
    let ext_lower = ext.to_lowercase();

    // First try to find it as a source format
    if let Some(cap) = all.iter().find(|c| c.from_ext == ext_lower) {
        return cap.category.clone();
    }

    // Then try as a target format
    if let Some(cap) = all.iter().find(|c| c.to_ext == ext_lower) {
        return cap.category.clone();
    }

    // Fallback to manual match for known categories that might not be in registry yet
    match ext_lower.as_str() {
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
        "csv" | "json" | "xlsx" | "xls" | "yaml" | "xml" | "toml" | "sql" | "sqlite" | "db"
        | "bib" | "ics" | "log" => FileCategory::Data,
        "zip" | "tar" | "gz" | "7z" | "rar" | "directory" => FileCategory::Archive,
        _ => FileCategory::Unknown,
    }
}

pub fn get_all_extensions() -> Vec<String> {
    let mut exts = std::collections::HashSet::new();
    let all = Registry::get_all_capabilities();
    for cap in all {
        exts.insert(cap.from_ext);
        exts.insert(cap.to_ext);
    }
    let mut list: Vec<String> = exts.into_iter().collect();
    list.sort();
    list
}

pub fn get_compatible_formats(input_ext: &str) -> Vec<FormatOption> {
    let caps = Registry::get_compatible_targets(input_ext);
    caps.into_iter()
        .map(|c| {
            let target_category = match c.to_ext.as_str() {
                "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" | "opus" => FileCategory::Audio,
                "webp" | "png" | "jpg" | "jpeg" | "ico" | "bmp" | "tiff" | "tga" => {
                    FileCategory::Image
                }
                "pdf" | "docx" | "md" | "html" | "txt" | "epub" | "rtf" | "odt" | "srt" => {
                    FileCategory::Document
                }
                "csv" | "json" | "xlsx" | "yaml" | "xml" | "toml" | "sql" => FileCategory::Data,
                "zip" | "folder" => FileCategory::Archive,
                _ => c.category.clone(),
            };
            let name = match &target_category {
                FileCategory::Video => format!("{} Media", c.to_ext.to_uppercase()),
                FileCategory::Audio => format!("{} Audio", c.to_ext.to_uppercase()),
                FileCategory::Image => format!("{} Graphics", c.to_ext.to_uppercase()),
                FileCategory::Document => format!("{} Document", c.to_ext.to_uppercase()),
                FileCategory::Data => format!("{} Data", c.to_ext.to_uppercase()),
                FileCategory::Archive => {
                    if c.to_ext == "folder" {
                        "Extract Archive".to_string()
                    } else {
                        "ZIP Archive".to_string()
                    }
                }
                _ => format!("{} Format", c.to_ext.to_uppercase()),
            };

            let description = match c.engine {
                crate::registry::ConversionEngine::FFmpeg => {
                    if c.to_ext == "srt" {
                        "FFmpeg embedded subtitle extraction".to_string()
                    } else if target_category == FileCategory::Audio {
                        format!("FFmpeg {} audio stream export", c.to_ext.to_uppercase())
                    } else {
                        format!("FFmpeg {} video encoding", c.to_ext.to_uppercase())
                    }
                }
                crate::registry::ConversionEngine::ImageMagick => {
                    format!("ImageMagick {} image encoding", c.to_ext.to_uppercase())
                }
                crate::registry::ConversionEngine::Pandoc => {
                    format!("Pandoc {} document export", c.to_ext.to_uppercase())
                }
                crate::registry::ConversionEngine::Poppler => {
                    format!("Poppler {} document extraction", c.to_ext.to_uppercase())
                }
                _ if target_category == FileCategory::Archive => {
                    if c.to_ext == "folder" {
                        "Extract all files from ZIP".to_string()
                    } else {
                        "High-performance ZIP compression".to_string()
                    }
                }
                crate::registry::ConversionEngine::RustNative => format!(
                    "Native structured {} serialization",
                    c.to_ext.to_uppercase()
                ),
                _ => format!("Convert to {}", c.to_ext),
            };

            FormatOption {
                extension: c.to_ext.clone(),
                name,
                category: target_category,
                subcategory: c.subcategory,
                description,
                comparison_note: c.fidelity_note,
                is_lossless: c.is_lossless,
                is_recommended: true,
                recommended_for: c.recommended_for,
                sidecar_engine: if c.from_ext == "pdf" && c.to_ext != "txt" {
                    "Poppler + Pandoc".to_string()
                } else {
                    c.engine.to_string()
                },
                pros: vec!["Fast".to_string()],
                cons: vec![],
            }
        })
        .collect()
}

pub fn get_smart_recommendations(input_ext: &str) -> Vec<FormatOption> {
    let category = get_category_for_extension(input_ext);
    let formats = get_compatible_formats(input_ext);

    let priority_list = match category {
        FileCategory::Video => vec!["mp4", "webm", "mp3", "gif"],
        FileCategory::Audio => vec!["mp3", "wav", "flac", "m4a"],
        FileCategory::Image | FileCategory::Vector => vec!["webp", "png", "jpg", "pdf"],
        FileCategory::Document => vec!["docx", "html", "txt", "md"],
        FileCategory::Data => vec!["json", "csv", "yaml", "xml"],
        FileCategory::Archive => vec!["zip", "folder"],
        _ => vec![],
    };

    let mut recommended = Vec::new();

    // First, pick from the priority list in order
    for ext in priority_list {
        if let Some(f) = formats.iter().find(|f| f.extension == ext) {
            recommended.push(f.clone());
        }
    }

    // If we have fewer than 4, fill with other recommended formats
    if recommended.len() < 4 {
        for f in formats {
            if f.is_recommended && !recommended.iter().any(|r| r.extension == f.extension) {
                recommended.push(f);
                if recommended.len() >= 4 {
                    break;
                }
            }
        }
    }

    recommended.truncate(4);
    recommended
}

pub fn get_quick_presets() -> Vec<QuickPreset> {
    // ...
    vec![
        QuickPreset {
            id: "p1".to_string(),
            title: "PDF to Editable DOCX".to_string(),
            target_name: "Word".to_string(),
            from_category: FileCategory::Document,
            to_format: "docx".to_string(),
            description: "Text and basic layout extraction".to_string(),
            badge: "Docs".to_string(),
            icon: "file-text".to_string(),
        },
        QuickPreset {
            id: "p2".to_string(),
            title: "Image to WebP".to_string(),
            target_name: "WebP".to_string(),
            from_category: FileCategory::Image,
            to_format: "webp".to_string(),
            description: "Smaller web-friendly images".to_string(),
            badge: "Images".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_media_targets_use_the_target_category() {
        let formats = get_compatible_formats("mp4");
        let mp3 = formats
            .iter()
            .find(|format| format.extension == "mp3")
            .unwrap();
        let srt = formats
            .iter()
            .find(|format| format.extension == "srt")
            .unwrap();
        assert_eq!(mp3.category, FileCategory::Audio);
        assert_eq!(srt.category, FileCategory::Document);
        assert!(srt.description.contains("subtitle extraction"));
    }

    #[test]
    fn pdf_editable_exports_report_every_required_engine() {
        let formats = get_compatible_formats("pdf");
        let docx = formats
            .iter()
            .find(|format| format.extension == "docx")
            .unwrap();
        let text = formats
            .iter()
            .find(|format| format.extension == "txt")
            .unwrap();
        assert_eq!(docx.sidecar_engine, "Poppler + Pandoc");
        assert_eq!(text.sidecar_engine, "Poppler (Poppler-utils)");
    }

    #[test]
    fn transcoding_a_lossy_source_is_not_labelled_lossless() {
        let formats = get_compatible_formats("mp3");
        assert!(
            !formats
                .iter()
                .find(|format| format.extension == "wav")
                .unwrap()
                .is_lossless
        );
        let images = get_compatible_formats("jpg");
        assert!(
            !images
                .iter()
                .find(|format| format.extension == "png")
                .unwrap()
                .is_lossless
        );
    }
}
