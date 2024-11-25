use std::fs;
use tempfile::NamedTempFile;
use std::io::Write;
use std::path::PathBuf;
use crate::utils::get_file_info;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temp_file(content: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content).unwrap();
        file.flush().unwrap();
        file
    }

    fn create_temp_file_with_extension(content: &[u8], extension: &str) -> (NamedTempFile, PathBuf) {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content).unwrap();
        file.flush().unwrap();
        let new_path = file.path().with_extension(extension);
        fs::rename(file.path(), &new_path).unwrap();
        (file, new_path)
    }

    #[test]
    fn test_typescript_react_detection() {
        let content = "import React from 'react';\nexport const App = () => <div>Hello</div>;";
        let (_file, path) = create_temp_file_with_extension(content.as_bytes(), "tsx");
        let info = get_file_info(&path).unwrap();
        assert!(info.is_text);
        assert_eq!(info.interpreter, Some("typescript-react".to_string()));
    }

    #[test]
    fn test_shell_script_detection() {
        let content = "#!/bin/bash\necho 'Hello'";
        let file = create_temp_file(content.as_bytes());
        let info = get_file_info(file.path()).unwrap();
        assert!(info.is_text);
        assert_eq!(info.interpreter, Some("bash".to_string()));
    }

    #[test]
    fn test_env_shebang_detection() {
        let content = "#!/usr/bin/env python3\nprint('Hello')";
        let file = create_temp_file(content.as_bytes());
        let info = get_file_info(file.path()).unwrap();
        assert!(info.is_text);
        assert_eq!(info.interpreter, Some("python3".to_string()));
    }

    #[test]
    fn test_binary_file_detection() {
        let content = vec![0x7F, 0x45, 0x4C, 0x46, 0x01, 0x01, 0x01, 0x00];
        let file = create_temp_file(&content);
        let info = get_file_info(file.path()).unwrap();
        assert!(!info.is_text);
        assert_eq!(info.interpreter, None);
    }

    #[test]
    fn test_dotfile_detection() {
        let content = "ignore_pattern = \"*.log\"";
        let file = create_temp_file(content.as_bytes());
        let path = file.path().with_file_name(".gitignore");
        fs::rename(file.path(), &path).unwrap();
        let info = get_file_info(&path).unwrap();
        assert!(info.is_text);
        assert_eq!(info.interpreter, Some("config".to_string()));
    }

    #[test]
    fn test_common_programming_languages() {
        let test_cases = vec![
            ("fn main() {}", "rs", "rust"),
            ("def hello(): pass", "py", "python3"),
            ("package main", "go", "go"),
            ("public class Test {}", "java", "java"),
            ("console.log('test');", "js", "javascript"),
        ];

        for (content, ext, expected_interpreter) in test_cases {
            let (_file, path) = create_temp_file_with_extension(content.as_bytes(), ext);
            let info = get_file_info(&path).unwrap();
            assert!(info.is_text, "File with extension {} should be text", ext);
            assert_eq!(info.interpreter, Some(expected_interpreter.to_string()), 
                      "File with extension {} should have interpreter {}", ext, expected_interpreter);
        }
    }

    #[test]
    fn test_various_config_files() {
        let test_cases = vec![
            ("{\"key\": \"value\"}", "json", "json"),
            ("key: value", "yaml", "yaml"),
            ("key = \"value\"", "toml", "toml"),
            ("[section]\nkey=value", "ini", "ini"),
        ];

        for (content, ext, expected_interpreter) in test_cases {
            let (_file, path) = create_temp_file_with_extension(content.as_bytes(), ext);
            let info = get_file_info(&path).unwrap();
            assert!(info.is_text, "File with extension {} should be text", ext);
            assert_eq!(info.interpreter, Some(expected_interpreter.to_string()));
        }
    }

    #[test]
    fn test_web_technology_files() {
        let test_cases = vec![
            ("<html><body></body></html>", "html", "html"),
            (".class { color: red; }", "css", "css"),
            ("$color: red;", "scss", "scss"),
            ("<template><div></div></template>", "vue", "vue"),
        ];

        for (content, ext, expected_interpreter) in test_cases {
            let (_file, path) = create_temp_file_with_extension(content.as_bytes(), ext);
            let info = get_file_info(&path).unwrap();
            assert!(info.is_text, "File with extension {} should be text", ext);
            assert_eq!(info.interpreter, Some(expected_interpreter.to_string()));
        }
    }

    #[test]
    fn test_empty_file() {
        let file = create_temp_file(&[]);
        let info = get_file_info(file.path()).unwrap();
        assert!(info.is_text);
        assert_eq!(info.size, 0);
    }

    #[test]
    fn test_text_with_binary_content() {
        let mut content = Vec::with_capacity(512);
        content.extend_from_slice(b"Hello World!\n");
        // Fill the rest with non-printable characters
        for i in content.len()..512 {
            content.push((i % 256) as u8);
        }
        let file = create_temp_file(&content);
        let info = get_file_info(file.path()).unwrap();
        assert!(!info.is_text);
    }

    #[test]
    fn test_extensionless_script_detection() {
        let content = "#!/usr/bin/env node\nconsole.log('Hello');";
        let file = create_temp_file(content.as_bytes());
        let info = get_file_info(file.path()).unwrap();
        assert!(info.is_text);
        assert_eq!(info.interpreter, Some("node".to_string()));
    }
}
