use std::path::Path;
use std::fs;
use anyhow::Result;
use humansize::{format_size as humansize_format, BINARY};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use std::io::{BufRead, BufReader};
use serde::Deserialize;
use log::{debug, trace};

#[derive(Debug)]
pub struct FileInfo {
    pub size: u64,
    pub is_text: bool,
    pub interpreter: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FileTypeConfig {
    file_type_multipliers: HashMap<String, f32>,
    mime_overrides: HashMap<String, String>,
    known_dotfiles: KnownDotfiles,
    binary_signatures: HashMap<String, Vec<u8>>,
    text_detection: TextDetection,
}

#[derive(Debug, Deserialize)]
struct KnownDotfiles {
    patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TextDetection {
    printable_ratio_threshold: f32,
    sample_size: usize,
}

static CONFIG: Lazy<FileTypeConfig> = Lazy::new(|| {
    let config_path = Path::new("config/filetypes.toml");
    let config_str = fs::read_to_string(config_path)
        .unwrap_or_else(|_| include_str!("../config/filetypes.toml").to_string());
    toml::from_str(&config_str).expect("Failed to parse file type configuration")
});

pub fn format_size(size: u64) -> String {
    humansize_format(size, BINARY)
}

fn normalize_shebang_line(line: &str) -> String {
    line.trim()
        .trim_start_matches("#!")
        .trim()
        .to_lowercase()
}

fn get_interpreter_for_extension(extension: &str) -> Option<String> {
    CONFIG.mime_overrides.get(extension)
        .map(|s| s.to_string())
}

fn has_shell_script_shebang(path: &Path) -> Option<String> {
    if let Ok(file) = fs::File::open(path) {
        if let Some(Ok(first_line)) = BufReader::new(file).lines().next() {
            if !first_line.trim().starts_with("#!") {
                return None;
            }

            let normalized = normalize_shebang_line(&first_line);
            
            // When handling env-based shebangs, extract the actual interpreter
            if normalized.contains("/env") {
                let parts: Vec<&str> = normalized.split_whitespace().collect();
                if let Some(&interpreter) = parts.get(1) {
                    return Some(interpreter.to_string());
                }
            }

            // Extract interpreter name from path
            if normalized.contains("/bin/") || normalized.contains("/usr/bin/") || 
               normalized.contains("/usr/local/bin/") {
                let parts: Vec<&str> = normalized.split('/').collect();
                if let Some(&interpreter) = parts.last() {
                    return Some(interpreter.to_string());
                }
            }

            // Direct interpreter name
            let parts: Vec<&str> = normalized.split_whitespace().collect();
            if let Some(&interpreter) = parts.first() {
                return Some(interpreter.to_string());
            }
        }
    }
    None
}

fn is_binary_file(path: &Path) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut buffer = vec![0u8; CONFIG.text_detection.sample_size];
        if let Ok(n) = std::io::Read::read(&mut file, &mut buffer) {
            trace!("Reading {} bytes for binary detection from {}", n, path.display());
            
            // Check for binary signatures
            for (name, signature) in &CONFIG.binary_signatures {
                if n >= signature.len() && buffer[..signature.len()] == signature[..] {
                    debug!("Detected binary file {} with signature {}", path.display(), name);
                    return true;
                }
            }

            // Content analysis
            let printable = buffer[..n].iter()
                .filter(|&&b| b.is_ascii_graphic() || b.is_ascii_whitespace())
                .count();
            let ratio = printable as f32 / n as f32;
            trace!("Printable ratio for {}: {}", path.display(), ratio);
            
            if ratio < CONFIG.text_detection.printable_ratio_threshold {
                debug!("File {} classified as binary based on content analysis", path.display());
                return true;
            }
        }
    }
    false
}

fn is_known_dotfile(path: &Path) -> bool {
    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
        return CONFIG.known_dotfiles.patterns.iter().any(|pattern| pattern == file_name);
    }
    false
}

pub fn get_file_info(path: &Path) -> Result<FileInfo> {
    debug!("Analyzing file: {}", path.display());
    let metadata = fs::metadata(path)?;
    
    // Perform binary detection first and cache the result
    let is_binary = is_binary_file(path);
    if is_binary {
        debug!("File type: binary");
        return Ok(FileInfo {
            size: metadata.len(),
            is_text: false,
            interpreter: None,
        });
    }

    // Check for shell script shebang
    if let Some(interpreter) = has_shell_script_shebang(path) {
        debug!("File type: {}", interpreter);
        return Ok(FileInfo {
            size: metadata.len(),
            is_text: true,
            interpreter: Some(interpreter),
        });
    }

    // Check for known dotfiles
    if is_known_dotfile(path) {
        debug!("File type: config");
        return Ok(FileInfo {
            size: metadata.len(),
            is_text: true,
            interpreter: Some("config".to_string()),
        });
    }
    
    // Get file extension and check for interpreter mapping
    if let Some(extension) = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
    {
        if let Some(interpreter) = get_interpreter_for_extension(&extension) {
            debug!("File type: {}", interpreter);
            return Ok(FileInfo {
                size: metadata.len(),
                is_text: true,
                interpreter: Some(interpreter),
            });
        }
    }
    
    debug!("File type: text");
    Ok(FileInfo {
        size: metadata.len(),
        is_text: true,
        interpreter: None,
    })
}
