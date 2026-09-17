use crate::formats::FileCategory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionCapability {
    pub from_ext: String,
    pub to_ext: String,
    pub category: FileCategory,
    pub engine: ConversionEngine,
    pub sidecar_requirement: Option<String>,
    pub subcategory: String,
    pub is_lossless: bool,
    pub recommended_for: Vec<String>,
    pub fidelity_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConversionEngine {
    FFmpeg,
    ImageMagick,
    Pandoc,
    Poppler, // pdftotext, pdftohtml
    RustNative,
    BrowserCanvas,
    BrowserJsPDF,
    BrowserText,
}

impl std::fmt::Display for ConversionEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ConversionEngine::FFmpeg => "FFmpeg",
            ConversionEngine::ImageMagick => "ImageMagick",
            ConversionEngine::Pandoc => "Pandoc",
            ConversionEngine::Poppler => "Poppler (Poppler-utils)",
            ConversionEngine::RustNative => "Rust-Native",
            ConversionEngine::BrowserCanvas => "Canvas-API",
            ConversionEngine::BrowserJsPDF => "jsPDF-Core",
            ConversionEngine::BrowserText => "Text-Engine",
        };
        write!(f, "{}", s)
    }
}

pub struct Registry;

impl Registry {
    pub fn get_all_capabilities() -> Vec<ConversionCapability> {
        let mut caps = Vec::new();

        // --- VIDEO ---
        let video_exts = ["mp4", "webm", "mkv", "mov", "avi", "flv", "wmv", "m4v", "ts", "3gp", "ogv", "vob"];
        let video_targets = ["mp4", "webm", "mkv", "mov", "avi", "gif", "mp3", "wav", "flac"];
        for &from in &video_exts {
            for &to in &video_targets {
                if from == to { continue; }
                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: FileCategory::Video,
                    engine: ConversionEngine::FFmpeg,
                    sidecar_requirement: Some("ffmpeg".to_string()),
                    subcategory: "Broadcast".to_string(),
                    is_lossless: false,
                    recommended_for: vec!["Universal Playback".to_string()],
                    fidelity_note: None,
                });
            }
        }

        // --- AUDIO ---
        let audio_exts = ["mp3", "wav", "flac", "aac", "ogg", "m4a", "opus", "wma", "aiff"];
        let audio_targets = ["mp3", "wav", "flac", "aac", "ogg", "m4a", "opus"];
        for &from in &audio_exts {
            for &to in &audio_targets {
                if from == to { continue; }
                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: FileCategory::Audio,
                    engine: ConversionEngine::FFmpeg,
                    sidecar_requirement: Some("ffmpeg".to_string()),
                    subcategory: "HiFi".to_string(),
                    is_lossless: ["flac", "wav"].contains(&to),
                    recommended_for: vec!["Music".to_string()],
                    fidelity_note: None,
                });
            }
        }

        // --- IMAGE / VECTOR ---
        let image_exts = ["png", "jpg", "jpeg", "webp", "avif", "bmp", "tiff", "ico", "heic", "tga", "psd", "svg", "eps", "ai"];
        let image_targets = ["webp", "avif", "png", "jpg", "ico", "bmp", "tiff", "tga", "pdf"];
        for &from in &image_exts {
            for &to in &image_targets {
                if from == to { continue; }
                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: FileCategory::Image,
                    engine: ConversionEngine::ImageMagick,
                    sidecar_requirement: Some("magick".to_string()),
                    subcategory: "Studio".to_string(),
                    is_lossless: ["png", "bmp", "tiff"].contains(&to),
                    recommended_for: vec!["Creative".to_string()],
                    fidelity_note: if ["svg", "eps", "ai"].contains(&from) {
                        Some("Requires external delegates (librsvg/Ghostscript)".to_string())
                    } else {
                        None
                    },
                });
            }
        }

        // --- DOCUMENTS ---
        let doc_exts = ["pdf", "docx", "doc", "md", "html", "txt", "epub", "rtf", "odt"];
        let doc_targets = ["pdf", "docx", "md", "html", "txt", "rtf", "epub", "odt"];
        for &from in &doc_exts {
            for &to in &doc_targets {
                if from == to { continue; }
                let engine = if from == "pdf" {
                    ConversionEngine::Poppler
                } else {
                    ConversionEngine::Pandoc
                };
                let sidecar = if from == "pdf" {
                    if to == "txt" { "pdftotext" } else { "pdftohtml" }
                } else {
                    "pandoc"
                };

                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: FileCategory::Document,
                    engine,
                    sidecar_requirement: Some(sidecar.to_string()),
                    subcategory: "Universal".to_string(),
                    is_lossless: true,
                    recommended_for: vec!["Editing".to_string()],
                    fidelity_note: if from == "pdf" { Some("Heuristic extraction, layout may shift".to_string()) } else { None },
                });
            }
        }

        // --- DATA ---
        let data_exts = ["csv", "json", "xlsx", "xls", "yaml", "xml", "toml", "sql", "sqlite", "db", "bib", "ics", "log"];
        let data_targets = ["json", "csv", "yaml", "toml", "xml", "xlsx", "sql"];
        for &from in &data_exts {
            for &to in &data_targets {
                if from == to { continue; }
                // Restricted data paths
                if (["log", "bib", "ics", "sqlite", "db"].contains(&from)) && to != "json" {
                    continue;
                }

                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: FileCategory::Data,
                    engine: ConversionEngine::RustNative,
                    sidecar_requirement: None,
                    subcategory: "Data Exchange".to_string(),
                    is_lossless: true,
                    recommended_for: vec!["Analysis".to_string()],
                    fidelity_note: None,
                });
            }
        }

        // --- ARCHIVE ---
        caps.push(ConversionCapability {
            from_ext: "zip".to_string(),
            to_ext: "folder".to_string(),
            category: FileCategory::Archive,
            engine: ConversionEngine::RustNative,
            sidecar_requirement: None,
            subcategory: "Compression".to_string(),
            is_lossless: true,
            recommended_for: vec!["Extraction".to_string()],
            fidelity_note: None,
        });

        let archivable = ["zip", "tar", "gz", "7z", "rar", "directory"];
        for from in archivable {
            if from == "directory" || from != "zip" {
                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: "zip".to_string(),
                    category: FileCategory::Archive,
                    engine: ConversionEngine::RustNative,
                    sidecar_requirement: None,
                    subcategory: "Compression".to_string(),
                    is_lossless: true,
                    recommended_for: vec!["Storage".to_string()],
                    fidelity_note: None,
                });
            }
        }

        caps
    }

    pub fn get_compatible_targets(input_ext: &str) -> Vec<ConversionCapability> {
        let all = Self::get_all_capabilities();
        let input_ext = input_ext.to_lowercase();
        all.into_iter()
            .filter(|c| c.from_ext == input_ext)
            .collect()
    }
}
