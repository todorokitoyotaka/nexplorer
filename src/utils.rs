use std::path::Path;
use std::fs;
use anyhow::Result;
use humansize::{format_size as humansize_format, BINARY};

#[derive(Debug)]
pub struct FileInfo {
    pub size: u64,
    pub is_text: bool,
}

pub fn format_size(size: u64) -> String {
    humansize_format(size, BINARY)
}

pub fn get_file_info(path: &Path) -> Result<FileInfo> {
    let metadata = fs::metadata(path)?;
    let mime_type = mime_guess::from_path(path).first_or_octet_stream();
    
    let is_text = mime_type.type_() == "text"
        || mime_type.subtype() == "javascript"
        || mime_type.subtype() == "json"
        || mime_type.subtype() == "xml"
        || mime_type.subtype() == "sql"
        || path.extension().map_or(false, |ext| ext == "sql");

    Ok(FileInfo {
        size: metadata.len(),
        is_text,
    })
}
