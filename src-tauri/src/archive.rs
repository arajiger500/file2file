//! Archives use portable names, bounded streaming and private extraction roots.
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};
use tokio::sync::watch;

const MAX_BYTES: u64 = 5 * 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 10_000;

pub fn portable_name(name: &str) -> Result<String, String> {
    if name.starts_with('/')
        || name.contains('\\')
        || name.contains(':')
        || name.chars().any(|c| c.is_control())
    {
        return Err(format!("Unsafe archive path: {name:?}"));
    }
    let name = name.trim_end_matches('/');
    let components: Vec<_> = name.split('/').filter(|c| *c != ".").collect();
    if components.is_empty() || components.len() > 64 {
        return Err("Invalid archive path depth".into());
    }
    for part in &components {
        let base = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if part.is_empty()
            || *part == ".."
            || part.ends_with(['.', ' '])
            || part.len() > 240
            || part.contains(['<', '>', '"', '|', '?', '*'])
            || ["CON", "PRN", "AUX", "NUL"].contains(&base.as_str())
            || ["COM", "LPT"].iter().any(|prefix| {
                base.strip_prefix(prefix).is_some_and(|suffix| {
                    matches!(
                        suffix,
                        "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                    )
                })
            })
        {
            return Err(format!(
                "Unsafe or non-portable archive component: {part:?}"
            ));
        }
    }
    Ok(components.join("/"))
}

struct Budget {
    bytes: u64,
    entries: usize,
    explicit: HashSet<String>,
    paths: HashMap<String, (String, bool)>,
    cancel: Option<watch::Receiver<bool>>,
}
impl Budget {
    fn new(cancel: Option<watch::Receiver<bool>>) -> Self {
        Self {
            bytes: 0,
            entries: 0,
            explicit: HashSet::new(),
            paths: HashMap::new(),
            cancel,
        }
    }
    fn check(&self) -> Result<(), String> {
        if self.cancel.as_ref().is_some_and(|c| *c.borrow()) {
            Err("Conversion cancelled".into())
        } else {
            Ok(())
        }
    }
    fn entry(&mut self, raw: &str, directory: bool) -> Result<String, String> {
        self.check()?;
        self.entries += 1;
        if self.entries > MAX_ENTRIES {
            return Err("Archive exceeds 10000 entries".into());
        }
        let name = portable_name(raw)?;
        if !self.explicit.insert(name.to_lowercase()) {
            return Err("Duplicate archive entry".into());
        }
        let parts: Vec<_> = name.split('/').collect();
        for end in 1..=parts.len() {
            let path = parts[..end].join("/");
            let is_dir = end < parts.len() || directory;
            let folded = path.to_lowercase();
            if let Some((original, old_dir)) = self.paths.get(&folded) {
                if original != &path || *old_dir != is_dir {
                    return Err("Archive path or case collision".into());
                }
            } else {
                self.paths.insert(folded, (path, is_dir));
            }
        }
        Ok(name)
    }
    fn copy(&mut self, reader: &mut impl Read, writer: &mut impl Write) -> Result<(), String> {
        let mut buffer = [0u8; 64 * 1024];
        loop {
            self.check()?;
            let n = reader.read(&mut buffer).map_err(|e| e.to_string())?;
            if n == 0 {
                return Ok(());
            }
            self.bytes = self
                .bytes
                .checked_add(n as u64)
                .ok_or("Archive size overflow")?;
            if self.bytes > MAX_BYTES {
                return Err("Archive exceeds 5 GiB expanded size".into());
            }
            writer.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
        }
    }
}

pub fn convert(
    input: &Path,
    output: &Path,
    from: &str,
    to: &str,
    cancel: Option<watch::Receiver<bool>>,
) -> Result<(), String> {
    let mut budget = Budget::new(cancel);
    if from == "zip" && to == "folder" {
        let mut zip = zip::ZipArchive::new(File::open(input).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if zip.len() > MAX_ENTRIES {
            return Err("Archive exceeds 10000 entries".into());
        }
        fs::create_dir(output).map_err(|e| e.to_string())?;
        for index in 0..zip.len() {
            let mut entry = zip.by_index(index).map_err(|e| e.to_string())?;
            let directory = entry.is_dir();
            if let Some(mode) = entry.unix_mode() {
                let kind = mode & 0o170000;
                if kind != 0 && kind != 0o100000 && kind != 0o040000 {
                    return Err("Archive links and special files are not supported".into());
                }
            }
            let name = budget.entry(entry.name(), directory)?;
            if entry.size() > MAX_BYTES.saturating_sub(budget.bytes) {
                return Err("Archive exceeds 5 GiB expanded size".into());
            }
            let dest = output.join(name);
            if directory {
                fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
            } else {
                fs::create_dir_all(dest.parent().ok_or("Missing entry parent")?)
                    .map_err(|e| e.to_string())?;
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&dest)
                    .map_err(|e| e.to_string())?;
                budget.copy(&mut entry, &mut file)?;
            }
        }
        return Ok(());
    }
    if to != "zip" {
        return Err("Unsupported archive route".into());
    }
    let mut zip = zip::ZipWriter::new(File::create(output).map_err(|e| e.to_string())?);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    match from {
        "folder" | "directory" => {
            for entry in walkdir::WalkDir::new(input)
                .follow_links(false)
                .min_depth(1)
                .max_depth(65)
                .sort_by_file_name()
            {
                budget.check()?;
                let entry = entry.map_err(|e| e.to_string())?;
                let meta = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
                if is_link(&meta) || (!meta.is_file() && !meta.is_dir()) {
                    return Err(
                        "Folders containing links, junctions or special files cannot be archived"
                            .into(),
                    );
                }
                let relative = entry
                    .path()
                    .strip_prefix(input)
                    .map_err(|e| e.to_string())?;
                let raw = relative
                    .components()
                    .map(|c| c.as_os_str().to_str().ok_or("Non-Unicode archive path"))
                    .collect::<Result<Vec<_>, _>>()?
                    .join("/");
                let name = budget.entry(&raw, meta.is_dir())?;
                if meta.is_dir() {
                    zip.add_directory(format!("{name}/"), options)
                        .map_err(|e| e.to_string())?;
                } else {
                    // Reject links again when opening, including a last-component replacement.
                    let mut open = fs::OpenOptions::new();
                    open.read(true);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        open.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
                    }
                    #[cfg(windows)]
                    {
                        use std::os::windows::fs::OpenOptionsExt;
                        open.custom_flags(0x00200000); // FILE_FLAG_OPEN_REPARSE_POINT
                    }
                    let mut file = open.open(entry.path()).map_err(|e| e.to_string())?;
                    let opened = file.metadata().map_err(|e| e.to_string())?;
                    if !opened.is_file() || is_link(&opened) {
                        return Err("Archive input changed type".into());
                    }
                    zip.start_file(name, options).map_err(|e| e.to_string())?;
                    budget.copy(&mut file, &mut zip)?;
                }
            }
        }
        "tar" => tar_to_zip(
            File::open(input).map_err(|e| e.to_string())?,
            &mut zip,
            &mut budget,
        )?,
        "gz" | "tgz" => {
            // Decode once with a real byte limit and CRC validation. Never reinterpret a damaged TAR as raw data.
            let mut decoded = tempfile::tempfile().map_err(|e| e.to_string())?;
            let mut decoder =
                flate2::read::MultiGzDecoder::new(File::open(input).map_err(|e| e.to_string())?);
            budget.copy(&mut decoder, &mut decoded)?;
            use std::io::{Seek, SeekFrom};
            decoded
                .seek(SeekFrom::Start(0))
                .map_err(|e| e.to_string())?;
            let mut header = [0u8; 512];
            let n = decoded.read(&mut header).map_err(|e| e.to_string())?;
            decoded
                .seek(SeekFrom::Start(0))
                .map_err(|e| e.to_string())?;
            let inner = input
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid GZIP filename")?;
            budget.bytes = 0;
            if from == "tgz"
                || inner.ends_with(".tar")
                || (n >= 262 && &header[257..262] == b"ustar")
            {
                tar_to_zip(decoded, &mut zip, &mut budget)?;
            } else {
                zip.start_file(budget.entry(inner, false)?, options)
                    .map_err(|e| e.to_string())?;
                budget.copy(&mut decoded, &mut zip)?;
            }
        }
        _ => return Err("Unsupported archive route".into()),
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

fn is_link(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}

fn tar_to_zip(
    reader: impl Read,
    zip: &mut zip::ZipWriter<File>,
    budget: &mut Budget,
) -> Result<(), String> {
    let mut tar = tar::Archive::new(reader);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);
    for entry in tar.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let kind = entry.header().entry_type();
        if !kind.is_file() && !kind.is_dir() {
            return Err("TAR links and special files are not supported".into());
        }
        let path = entry.path().map_err(|e| e.to_string())?;
        let raw = path.to_str().ok_or("Non-Unicode TAR path")?;
        if kind.is_dir() && matches!(raw, "." | "./") {
            continue;
        }
        let name = budget.entry(raw, kind.is_dir())?;
        if entry.size() > MAX_BYTES.saturating_sub(budget.bytes) {
            return Err("Archive exceeds 5 GiB expanded size".into());
        }
        if kind.is_dir() {
            zip.add_directory(format!("{name}/"), options)
                .map_err(|e| e.to_string())?;
        } else {
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            budget.copy(&mut entry, zip)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_paths_and_collisions() {
        for path in [
            "../x",
            "/x",
            "C:/x",
            "C:x",
            "\\\\server\\x",
            "a\\..\\x",
            "a/../x",
            "NUL.txt",
            "COM¹.txt",
            "lpt9",
            "a.",
            "a ",
            "a//b",
            "a:x",
        ] {
            assert!(portable_name(path).is_err(), "{path}");
        }
        assert_eq!(
            portable_name("./café/hello world.txt").unwrap(),
            "café/hello world.txt"
        );
        let mut budget = Budget::new(None);
        budget.entry("A/x", false).unwrap();
        assert!(budget.entry("a/y", false).is_err());
        assert!(budget.entry("A/x", false).is_err());
        assert!(budget.entry("A/x/y", false).is_err());
    }
    #[test]
    fn real_zip_traversal_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("bad.zip");
        let mut zip = zip::ZipWriter::new(File::create(&input).unwrap());
        zip.start_file("../escape", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"evil").unwrap();
        zip.finish().unwrap();
        assert!(convert(&input, &dir.path().join("out"), "zip", "folder", None).is_err());
        assert!(!dir.path().join("escape").exists());
    }
    #[test]
    fn byte_limit_measures_stream_and_cancellation() {
        let mut budget = Budget::new(None);
        budget.bytes = MAX_BYTES - 1;
        assert!(budget.copy(&mut &b"xx"[..], &mut Vec::new()).is_err());
        let (_, rx) = watch::channel(true);
        assert!(Budget::new(Some(rx)).check().is_err());
    }
    #[cfg(unix)]
    #[test]
    fn folder_links_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        fs::create_dir(&root).unwrap();
        std::os::unix::fs::symlink(&root, root.join("loop")).unwrap();
        assert!(convert(&root, &dir.path().join("out.zip"), "folder", "zip", None).is_err());
    }
}
