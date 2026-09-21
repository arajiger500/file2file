use crate::converter::CollisionPolicy;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

fn rename_directory_noreplace(source: &Path, destination: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::{ffi::CString, os::unix::ffi::OsStrExt};
        let from = CString::new(source.as_os_str().as_bytes())?;
        let to = CString::new(destination.as_os_str().as_bytes())?;
        #[cfg(target_os = "linux")]
        let result = unsafe {
            libc::renameat2(
                libc::AT_FDCWD,
                from.as_ptr(),
                libc::AT_FDCWD,
                to.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        #[cfg(target_os = "macos")]
        let result = unsafe { libc::renamex_np(from.as_ptr(), to.as_ptr(), libc::RENAME_EXCL) };
        if result == 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
    #[cfg(windows)]
    {
        fs::rename(source, destination)
    } // Windows refuses to replace a directory.
}

/// Publish a completed artifact in the same filesystem. Never copy over a live destination.
pub fn publish(
    source: &Path,
    root: &Path,
    stem: &str,
    extension: &str,
    policy: &CollisionPolicy,
) -> Result<(PathBuf, bool), String> {
    let metadata = fs::symlink_metadata(source).map_err(|e| format!("Missing output: {e}"))?;
    let directory = extension == "folder";
    if metadata.file_type().is_symlink()
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.is_file())
    {
        return Err("Engine output has an invalid file type".into());
    }
    if !directory {
        if metadata.len() == 0 {
            return Err("Conversion produced an empty file".into());
        }
        fs::File::open(source)
            .and_then(|f| f.sync_all())
            .map_err(|e| format!("Cannot flush output: {e}"))?;
    }
    for index in 0..10_000 {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("_{index}")
        };
        let name = if directory {
            format!("{stem}_extracted{suffix}")
        } else {
            format!("{stem}_converted{suffix}.{extension}")
        };
        let dest = root.join(name);
        if matches!(policy, CollisionPolicy::Overwrite) {
            if directory {
                return Err("Replacing extracted folders is disabled. Choose auto-rename to preserve existing contents.".into());
            }
            if let Ok(meta) = fs::symlink_metadata(&dest) {
                if !meta.is_file() || meta.file_type().is_symlink() {
                    return Err("Refusing to overwrite a symlink or non-regular output".into());
                }
            }
            // TempPath::persist atomically replaces the directory entry, never follows its symlink.
            tempfile::TempPath::try_from_path(source.to_path_buf())
                .map_err(|e| e.to_string())?
                .persist(&dest)
                .map_err(|e| format!("Cannot publish output: {e}"))?;
            return Ok((dest, false));
        }
        let result = if directory {
            rename_directory_noreplace(source, &dest)
        } else {
            fs::hard_link(source, &dest)
        };
        match result {
            Ok(()) => return Ok((dest, false)),
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => match policy {
                CollisionPolicy::AutoRename => continue,
                CollisionPolicy::Skip => return Ok((dest, true)),
                _ => {
                    return Err(
                        "Output already exists. Choose auto-rename, skip or overwrite.".into(),
                    )
                }
            },
            Err(e) => return Err(format!("Cannot publish output safely: {e}")),
        }
    }
    Err("Too many output name collisions".into())
}

pub fn copy_new(source: &Path, destination: &Path) -> Result<(), String> {
    if !source.is_file() {
        return Err("Source must be a regular file".into());
    }
    let parent = destination
        .parent()
        .ok_or("Destination must have a parent directory")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let destination = parent.join(
        destination
            .file_name()
            .ok_or("Missing destination filename")?,
    );
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    let mut input = fs::File::open(source).map_err(|e| e.to_string())?;
    io::copy(&mut input, &mut temporary).map_err(|e| e.to_string())?;
    temporary.as_file().sync_all().map_err(|e| e.to_string())?;
    temporary
        .persist_noclobber(destination)
        .map_err(|e| format!("Cannot export without overwriting: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concurrent_publication_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("work");
        fs::write(&source, "complete").unwrap();
        std::thread::scope(|scope| {
            let handles: Vec<_> = (0..12)
                .map(|_| {
                    scope.spawn(|| {
                        publish(
                            &source,
                            dir.path(),
                            "name",
                            "txt",
                            &CollisionPolicy::AutoRename,
                        )
                        .unwrap()
                        .0
                    })
                })
                .collect();
            let paths: std::collections::HashSet<_> =
                handles.into_iter().map(|h| h.join().unwrap()).collect();
            assert_eq!(paths.len(), 12);
        });
        assert_eq!(
            fs::read_to_string(dir.path().join("name_converted.txt")).unwrap(),
            "complete"
        );
    }
    #[test]
    fn extraction_never_replaces_an_existing_directory() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("work");
        fs::create_dir(&source).unwrap();
        fs::create_dir(dir.path().join("name_extracted")).unwrap();
        assert!(publish(&source, dir.path(), "name", "folder", &CollisionPolicy::Ask).is_err());
        assert!(source.exists());
        assert!(publish(
            &source,
            dir.path(),
            "name",
            "folder",
            &CollisionPolicy::Overwrite
        )
        .is_err());
        assert!(publish(
            &source,
            dir.path(),
            "name",
            "folder",
            &CollisionPolicy::AutoRename
        )
        .unwrap()
        .0
        .ends_with("name_extracted_1"));
    }
    #[cfg(unix)]
    #[test]
    fn dangling_symlink_and_export_do_not_clobber() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("work");
        fs::write(&source, "new").unwrap();
        let victim = dir.path().join("victim");
        let destination = dir.path().join("name_converted.txt");
        std::os::unix::fs::symlink(&victim, &destination).unwrap();
        assert!(publish(
            &source,
            dir.path(),
            "name",
            "txt",
            &CollisionPolicy::Overwrite
        )
        .is_err());
        assert!(copy_new(&source, &destination).is_err());
        assert!(!victim.exists());
    }
}
