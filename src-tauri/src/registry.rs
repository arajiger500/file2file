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
    pub limitations: Vec<String>,
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
        let video_exts = [
            "mp4", "webm", "mkv", "mov", "avi", "flv", "wmv", "m4v", "ts", "3gp", "ogv", "vob",
        ];
        let video_targets = [
            "mp4", "webm", "mkv", "mov", "avi", "gif", "mp3", "wav", "flac", "srt",
        ];
        for &from in &video_exts {
            for &to in &video_targets {
                if from == to {
                    continue;
                }
                let is_audio = ["mp3", "wav", "flac"].contains(&to);
                let is_gif = to == "gif";
                let is_subtitle = to == "srt";
                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: FileCategory::Video,
                    engine: ConversionEngine::FFmpeg,
                    sidecar_requirement: Some("ffmpeg".to_string()),
                    subcategory: if is_audio {
                        "Audio Extract".to_string()
                    } else if is_gif {
                        "Animation".to_string()
                    } else if is_subtitle {
                        "Subtitle Extract".to_string()
                    } else {
                        "Video Transcode".to_string()
                    },
                    is_lossless: false,
                    recommended_for: if is_audio {
                        vec!["Audio Track Extraction".to_string()]
                    } else if is_gif {
                        vec!["Social Media Clip".to_string()]
                    } else if is_subtitle {
                        vec!["Transcription / CC".to_string()]
                    } else {
                        vec!["Playback".to_string()]
                    },
                    fidelity_note: if is_gif {
                        Some("High-quality palette optimization applied".to_string())
                    } else if is_subtitle {
                        Some("Extracts the first subtitle track (if present)".to_string())
                    } else {
                        None
                    },
                    limitations: if is_gif {
                        vec!["No audio in GIF".to_string()]
                    } else if is_subtitle {
                        vec!["Fails if input has no internal subtitle tracks".to_string()]
                    } else {
                        vec!["Lossy compression".to_string()]
                    },
                });
            }
        }

        // --- AUDIO ---
        let audio_exts = [
            "mp3", "wav", "flac", "aac", "ogg", "m4a", "opus", "wma", "aiff",
        ];
        let audio_targets = ["mp3", "wav", "flac", "aac", "ogg", "m4a", "opus"];
        for &from in &audio_exts {
            for &to in &audio_targets {
                if from == to {
                    continue;
                }
                let lossless = ["flac", "wav"].contains(&to);
                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: FileCategory::Audio,
                    engine: ConversionEngine::FFmpeg,
                    sidecar_requirement: Some("ffmpeg".to_string()),
                    subcategory: if lossless {
                        "Lossless Audio".to_string()
                    } else {
                        "Compressed Audio".to_string()
                    },
                    is_lossless: lossless,
                    recommended_for: if lossless {
                        vec!["Archival".to_string(), "Master Quality".to_string()]
                    } else {
                        vec!["Everyday Listening".to_string(), "Portability".to_string()]
                    },
                    fidelity_note: None,
                    limitations: vec![],
                });
            }
        }

        // --- IMAGE / VECTOR ---
        let image_exts = [
            "png", "jpg", "jpeg", "webp", "bmp", "tiff", "ico", "tga", "psd", "svg",
        ];
        let image_targets = ["webp", "png", "jpg", "ico", "bmp", "tiff", "tga", "pdf"];
        for &from in &image_exts {
            for &to in &image_targets {
                if from == to {
                    continue;
                }
                if from == "jpeg" && to == "jpg" {
                    continue;
                }
                if from == "jpg" && to == "jpeg" {
                    continue;
                }

                let is_raw = ["raw", "cr2", "nef", "arw", "dng"].contains(&from);
                let engine = ConversionEngine::ImageMagick;
                let sidecar = "magick";

                let lossless = ["png", "bmp", "tiff"].contains(&to);
                let is_vector = from == "svg";
                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: if is_vector {
                        FileCategory::Vector
                    } else {
                        FileCategory::Image
                    },
                    engine,
                    sidecar_requirement: Some(sidecar.to_string()),
                    subcategory: if is_vector {
                        "Vector Rasterization".to_string()
                    } else if is_raw {
                        "RAW Processing".to_string()
                    } else {
                        "Image Processing".to_string()
                    },
                    is_lossless: lossless,
                    recommended_for: if to == "webp" {
                        vec!["Web Optimization".to_string()]
                    } else if to == "ico" {
                        vec!["Application Icons".to_string()]
                    } else if to == "pdf" {
                        vec!["Print / Sharing".to_string()]
                    } else {
                        vec!["General Use".to_string()]
                    },
                    fidelity_note: if is_raw {
                        Some(
                            "Extracts embedded preview or decodes via secondary engine".to_string(),
                        )
                    } else if to == "ico" {
                        Some("Generates multi-resolution icon (16px to 256px)".to_string())
                    } else if to == "jpg" {
                        Some("Transparency is flattened to white background".to_string())
                    } else {
                        None
                    },
                    limitations: vec![],
                });
            }
        }

        // --- DOCUMENTS ---
        let doc_exts = ["pdf", "docx", "md", "html", "txt", "epub", "rtf", "odt"];
        // Target format does NOT include pdf for Pandoc conversions because pandoc requires LaTeX/typst.
        let doc_targets = ["docx", "md", "html", "txt", "rtf", "epub", "odt"];
        for &from in &doc_exts {
            for &to in &doc_targets {
                if from == to {
                    continue;
                }
                let engine = if from == "pdf" {
                    ConversionEngine::Poppler
                } else {
                    ConversionEngine::Pandoc
                };
                let sidecar = if from == "pdf" {
                    if to == "txt" {
                        "pdftotext"
                    } else {
                        "pdftohtml"
                    }
                } else {
                    "pandoc"
                };

                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: FileCategory::Document,
                    engine,
                    sidecar_requirement: Some(sidecar.to_string()),
                    subcategory: if from == "pdf" {
                        "PDF Extraction".to_string()
                    } else {
                        "Document Interchange".to_string()
                    },
                    is_lossless: false,
                    recommended_for: vec!["Editing".to_string(), "Publishing".to_string()],
                    fidelity_note: if from == "pdf" {
                        Some("Heuristic extraction; layout and typography may shift".to_string())
                    } else {
                        None
                    },
                    limitations: if from == "pdf" {
                        vec!["Complex multi-column layouts may require manual adjustment"
                            .to_string()]
                    } else {
                        vec![]
                    },
                });
            }
        }

        // --- DATA ---
        let data_exts = [
            "csv", "json", "xlsx", "yaml", "xml", "toml", "sqlite", "db", "bib", "ics", "log",
        ];
        let data_targets = ["json", "csv", "yaml", "toml", "xml", "xlsx", "sql"];
        for &from in &data_exts {
            for &to in &data_targets {
                if from == to {
                    continue;
                }
                // Restricted data paths: these only export to json
                if ["log", "bib", "ics", "sqlite", "db"].contains(&from) && to != "json" {
                    continue;
                }
                // Only csv and sqlite/db can export to sql
                if to == "sql" && from != "csv" {
                    continue;
                }

                caps.push(ConversionCapability {
                    from_ext: from.to_string(),
                    to_ext: to.to_string(),
                    category: FileCategory::Data,
                    engine: ConversionEngine::RustNative,
                    sidecar_requirement: None,
                    subcategory: "Data Transformation".to_string(),
                    is_lossless: false,
                    recommended_for: vec!["Analysis".to_string(), "Interoperability".to_string()],
                    fidelity_note: None,
                    limitations: vec![],
                });
            }
        }

        // --- ARCHIVE ---
        // Extraction: zip to folder
        caps.push(ConversionCapability {
            from_ext: "zip".to_string(),
            to_ext: "folder".to_string(),
            category: FileCategory::Archive,
            engine: ConversionEngine::RustNative,
            sidecar_requirement: None,
            subcategory: "Archive Unpack".to_string(),
            is_lossless: true,
            recommended_for: vec!["Extraction".to_string()],
            fidelity_note: None,
            limitations: vec![],
        });

        // Compression: tar, gz, tgz, folder/directory to zip
        let archivable = ["tar", "gz", "tgz", "directory", "folder"];
        for from in archivable {
            caps.push(ConversionCapability {
                from_ext: from.to_string(),
                to_ext: "zip".to_string(),
                category: FileCategory::Archive,
                engine: ConversionEngine::RustNative,
                sidecar_requirement: None,
                subcategory: "Archive Packaging".to_string(),
                is_lossless: true,
                recommended_for: vec!["Portability".to_string(), "Storage".to_string()],
                fidelity_note: None,
                limitations: vec![],
            });
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

    pub fn is_supported(input_ext: &str, target_ext: &str) -> bool {
        let from = input_ext.to_lowercase();
        let to = target_ext.to_lowercase();
        Self::get_compatible_targets(&from)
            .iter()
            .any(|c| c.to_ext == to)
    }
}
