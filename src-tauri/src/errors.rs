pub fn map_technical_error(error: &str, input_path: &str, target_format: &str) -> String {
    let err_lower = error.to_lowercase();

    // FFmpeg specific errors
    if err_lower.contains("output file does not contain any stream") {
        return format!("This file seems to have no tracks that can be converted to {}. For example, you cannot extract audio from a silent video.", target_format.to_uppercase());
    }
    if err_lower.contains("invalid data found when processing input") {
        return "The input file appears to be corrupted or in an unsupported format.".to_string();
    }
    if err_lower.contains("permission denied") {
        return "Permission denied. Please check if the application has access to the input and output directories.".to_string();
    }
    if err_lower.contains("no space left on device") {
        return "Insufficient disk space. Please free up some space and try again.".to_string();
    }
    if err_lower.contains("codec not found") || err_lower.contains("unknown encoder") {
        return format!("Your version of FFmpeg doesn't support the {} encoder. Try updating or choosing a different format.", target_format.to_uppercase());
    }
    if err_lower.contains("moov atom not found") {
        return "The video file is incomplete or corrupted (missing metadata index).".to_string();
    }

    // Pandoc specific errors
    if err_lower.contains("pdflatex not found") {
        return "PDF generation requires a LaTeX installation (like MiKTeX or TeX Live).".to_string();
    }
    if err_lower.contains("could not find data file") {
        return "Pandoc could not find a required resource file for this conversion.".to_string();
    }

    // ImageMagick specific errors
    if err_lower.contains("no decode delegate for this image format") {
        return format!("ImageMagick does not support decoding this specific format on your system.");
    }

    // Generic fallbacks
    if err_lower.contains("failed to create") && err_lower.contains("sidecar") {
        let utility = if err_lower.contains("ffmpeg") { "FFmpeg" }
                     else if err_lower.contains("magick") { "ImageMagick" }
                     else if err_lower.contains("pandoc") { "Pandoc" }
                     else if err_lower.contains("pdftotext") { "Poppler" }
                     else { "a required utility" };
        return format!("{} is missing or could not be started. Please ensure it is correctly bundled or installed.", utility);
    }

    // If no specific match, return a cleaned up version or the original if it looks okay
    if error.len() > 200 {
        return "An unexpected error occurred during conversion. The file might be corrupted or the settings are incompatible.".to_string();
    }

    error.to_string()
}
