use serde::Serialize;
use std::{collections::HashSet, path::PathBuf};

#[derive(Serialize)]
pub struct InputFile {
    path: String,
    name: String,
    size: u64,
    is_directory: bool,
}

#[tauri::command]
pub async fn scan_inputs(paths: Vec<String>) -> Result<Vec<InputFile>, String> {
    if paths.len() > 256 {
        return Err("Choose at most 256 inputs at a time".into());
    }
    tokio::task::spawn_blocking(move || {
        let mut files = Vec::new();
        let mut seen = HashSet::new();
        let mut visited = 0;
        for path in paths {
            let root = PathBuf::from(path)
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if root.is_dir() {
                let name = root
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or("Invalid folder name")?
                    .to_owned();
                files.push(InputFile {
                    path: root.to_string_lossy().into(),
                    name,
                    size: 0,
                    is_directory: true,
                });
                continue;
            }
            for entry in walkdir::WalkDir::new(root)
                .follow_links(false)
                .max_depth(65)
            {
                visited += 1;
                if visited > 10_000 {
                    return Err("Folder scan exceeds 10000 entries".into());
                }
                let entry = entry.map_err(|e| e.to_string())?;
                let meta = std::fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err("Folder contains a junction or link".into());
                    }
                }
                if meta.file_type().is_symlink() {
                    return Err("Folder contains a symbolic link".into());
                }
                if entry.depth() > 64 {
                    return Err("Folder scan exceeds 64 levels".into());
                }
                if !meta.is_file() {
                    continue;
                }
                let path = entry
                    .path()
                    .to_str()
                    .ok_or("Non-Unicode input path")?
                    .to_owned();
                if !seen.insert(path.clone()) {
                    continue;
                }
                if files.len() >= 256 {
                    return Err("Folder exceeds 256 files; select a smaller batch".into());
                }
                files.push(InputFile {
                    name: entry.file_name().to_string_lossy().into(),
                    path,
                    size: meta.len(),
                    is_directory: false,
                });
            }
        }
        Ok(files)
    })
    .await
    .map_err(|e| e.to_string())?
}
