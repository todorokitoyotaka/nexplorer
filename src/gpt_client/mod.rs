use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use ignore::gitignore::{GitignoreBuilder, Gitignore};
use crate::utils;

const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1MB
const MODEL: &str = "gpt-4o-mini";
const CACHE_DIR: &str = ".cache";

// File size thresholds for smart summary lengths
const TINY_FILE_SIZE: u64 = 1024; // 1KB
const SMALL_FILE_SIZE: u64 = 10 * 1024; // 10KB
const MEDIUM_FILE_SIZE: u64 = 100 * 1024; // 100KB
const LARGE_FILE_SIZE: u64 = 500 * 1024; // 500KB

// Base summary length targets for different file sizes (English)
const TINY_SUMMARY_LENGTH: u32 = 75;     // Increased base length
const SMALL_SUMMARY_LENGTH: u32 = 150;   // Increased base length
const MEDIUM_SUMMARY_LENGTH: u32 = 250;  // Increased base length
const LARGE_SUMMARY_LENGTH: u32 = 350;   // Increased base length
const VERY_LARGE_SUMMARY_LENGTH: u32 = 500; // Increased base length

// File type specific length multipliers
const FILE_TYPE_MULTIPLIERS: &[(&str, f32)] = &[
    // Documentation files get longer summaries
    ("md", 1.5),    // Markdown files need detailed summaries
    ("txt", 1.3),   // Text files
    ("rst", 1.5),   // ReStructuredText
    ("adoc", 1.5),  // AsciiDoc
    ("pdf", 1.5),   // PDF documentation
    ("doc", 1.5),   // Word documents
    ("docx", 1.5),  // Word documents (newer format)
    
    // Source code files get moderate summaries
    ("rs", 1.2),    // Rust files
    ("py", 1.2),    // Python files
    ("js", 1.2),    // JavaScript files
    ("ts", 1.2),    // TypeScript files
    ("tsx", 1.2),   // TypeScript React files
    ("java", 1.2),  // Java files
    ("cpp", 1.2),   // C++ files
    ("c", 1.2),     // C files
    ("go", 1.2),    // Go files
    ("rb", 1.2),    // Ruby files
    ("php", 1.2),   // PHP files
    ("scala", 1.2), // Scala files
    ("swift", 1.2), // Swift files
    ("kt", 1.2),    // Kotlin files
    
    // Shell scripts
    ("sh", 1.2),    // Shell scripts
    ("bash", 1.2),  // Bash scripts
    ("zsh", 1.2),   // Zsh scripts
    ("fish", 1.2),  // Fish scripts
    
    // Web-related files
    ("html", 1.1),  // HTML files
    ("css", 1.1),   // CSS files
    ("scss", 1.1),  // SCSS files
    ("sass", 1.1),  // Sass files
    ("less", 1.1),  // Less files
    ("jsx", 1.2),   // React JSX files
    ("vue", 1.2),   // Vue files
    ("svelte", 1.2),// Svelte files
    
    // Configuration files need detailed summaries
    ("json", 1.3),  // JSON files
    ("yaml", 1.3),  // YAML files
    ("yml", 1.3),   // YML files
    ("toml", 1.3),  // TOML files
    ("ini", 1.2),   // INI files
    ("env", 1.2),   // Environment files
    ("conf", 1.2),  // Configuration files
    ("config", 1.2),// Configuration files
    
    // Database files
    ("sql", 1.3),   // SQL files
    ("pgsql", 1.3), // PostgreSQL files
    ("mysql", 1.3), // MySQL files
    
    // Build and package files
    ("xml", 1.2),   // XML files
    ("gradle", 1.2),// Gradle build files
    ("pom", 1.2),   // Maven POM files
    ("lock", 1.1),  // Lock files
    ("cargo", 1.2), // Cargo.toml gets special handling
    
    // Default multiplier for unknown types
    ("*", 1.0),     // Default multiplier
];

// Japanese language multiplier (approximately 50% more words needed)
const JAPANESE_MULTIPLIER: f32 = 1.5;

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatCompletion {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    content_hash: String,
    summary: String,
    timestamp: u64,
    language: String,
    summary_length: String,
}

pub enum BatchResult {
    Summaries(HashMap<String, String>),
    Answer(String),
}

pub struct GPTClient {
    api_key: String,
    collected_contents: Mutex<Vec<(String, String)>>,
    cache_dir: PathBuf,
    max_tokens: u32,
    summary_length: String,
    language: String,
    force_update: bool,
    gitignore: Option<Gitignore>,
    custom_patterns: Option<Vec<String>>,
    smart_length: bool,
}

impl GPTClient {
    pub fn new(summary_length: &str, language: &str, force_update: bool, custom_ignore: Option<&str>) -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable is not set")?;
            
        // Check if we should use smart length or fixed length
        let (smart_length, max_tokens) = if summary_length == "smart" {
            (true, MEDIUM_SUMMARY_LENGTH) // Default to medium for initial setup
        } else if let Ok(custom_length) = summary_length.parse::<u32>() {
            (false, custom_length)
        } else {
            (false, match summary_length {
                "short" => 50,
                "long" => 200,
                "super" => 500,
                _ => 100, // medium or any other value
            })
        };

        // Create cache directory if it doesn't exist
        let cache_dir = PathBuf::from(CACHE_DIR);
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        // Initialize gitignore
        let mut builder = GitignoreBuilder::new(".");
        if Path::new(".gitignore").exists() {
            builder.add(".gitignore");
        }
        let gitignore = builder.build().ok();

        // Parse custom ignore patterns
        let custom_patterns = custom_ignore.map(|patterns| {
            patterns.split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>()
        });

        Ok(Self {
            api_key,
            collected_contents: Mutex::new(Vec::new()),
            cache_dir,
            max_tokens,
            summary_length: summary_length.to_string(),
            language: language.to_string(),
            force_update,
            gitignore,
            custom_patterns,
            smart_length,
        })
    }

    fn get_file_type_multiplier(&self, path: &Path) -> f32 {
        // Only check by extension for supported file types
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| {
                FILE_TYPE_MULTIPLIERS
                    .iter()
                    .find(|&&(file_type, _)| file_type == ext)
                    .map(|(_, multiplier)| *multiplier)
                    .unwrap_or(1.0)
            })
            .unwrap_or(1.0)
    }

    fn calculate_summary_length(&self, file_size: u64, path: &Path) -> u32 {
        if !self.smart_length {
            return self.max_tokens;
        }

        let base_length = match file_size {
            size if size <= TINY_FILE_SIZE => TINY_SUMMARY_LENGTH,
            size if size <= SMALL_FILE_SIZE => SMALL_SUMMARY_LENGTH,
            size if size <= MEDIUM_FILE_SIZE => MEDIUM_SUMMARY_LENGTH,
            size if size <= LARGE_FILE_SIZE => LARGE_SUMMARY_LENGTH,
            _ => VERY_LARGE_SUMMARY_LENGTH,
        };

        // Apply file type multiplier
        let file_type_multiplier = self.get_file_type_multiplier(path);
        let length_with_type = (base_length as f32 * file_type_multiplier) as u32;

        // Apply Japanese language multiplier if needed
        if self.language.to_lowercase() == "japanese" {
            (length_with_type as f32 * JAPANESE_MULTIPLIER) as u32
        } else {
            length_with_type
        }
    }

    fn calculate_content_hash(&self, content: &str, query: Option<&str>) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        query.unwrap_or("").hash(&mut hasher);
        self.summary_length.hash(&mut hasher);
        self.language.hash(&mut hasher);
        
        format!("{:x}", hasher.finish())
    }

    fn get_cache_path(&self, content_hash: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", content_hash))
    }

    fn get_from_cache(&self, content_hash: &str) -> Option<String> {
        if self.force_update {
            return None; // Skip cache when update flag is set
        }

        let cache_path = self.get_cache_path(content_hash);
        if !cache_path.exists() {
            return None;
        }

        match fs::read_to_string(&cache_path) {
            Ok(cache_str) => {
                match serde_json::from_str::<CacheEntry>(&cache_str) {
                    Ok(entry) => {
                        // Validate cache entry matches current settings
                        if entry.language == self.language && entry.summary_length == self.summary_length {
                            Some(entry.summary)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    fn add_to_cache(&self, content_hash: String, summary: String) -> Result<()> {
        let cache_entry = CacheEntry {
            content_hash,
            summary,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs(),
            language: self.language.clone(),
            summary_length: self.summary_length.clone(),
        };

        let cache_path = self.get_cache_path(&cache_entry.content_hash);
        fs::write(
            cache_path,
            serde_json::to_string_pretty(&cache_entry)?,
        )?;

        Ok(())
    }

    pub async fn summarize_file(&self, path: &Path, custom_query: Option<&str>) -> Result<Option<String>> {
        if self.should_ignore(path) {
            return Ok(None);
        }

        let file_info = utils::get_file_info(path)?;
        if !file_info.is_text {
            eprintln!("File type detected as binary: {}", path.display());
            return Ok(None);
        }

        if file_info.size > MAX_FILE_SIZE {
            return Ok(Some("File too large for summarization".to_string()));
        }

        let content = fs::read_to_string(path)?;
        if content.trim().is_empty() {
            return Ok(Some("Empty file".to_string()));
        }

        let content_hash = self.calculate_content_hash(&content, custom_query);

        // Check cache first
        if let Some(cached_summary) = self.get_from_cache(&content_hash) {
            return Ok(Some(cached_summary));
        }

        // Calculate appropriate summary length based on file size and type
        let summary_length = self.calculate_summary_length(file_info.size, path);
        
        // Generate new summary with dynamic length
        let summary = self.get_gpt_summary(&content, custom_query, summary_length).await?;

        // Add to cache
        self.add_to_cache(content_hash, summary.clone())?;
        Ok(Some(summary))
    }

    async fn get_gpt_summary(&self, content: &str, custom_prompt: Option<&str>, summary_length: u32) -> Result<String> {
        let prompt = if let Some(query) = custom_prompt {
            format!("{} (respond in {})

{}", query, self.language, content)
        } else {
            format!(
                "Provide a detailed summary of the following file content in approximately {} words in {}.                 Focus on its main purpose, key elements, and important details:

{}",
                summary_length, self.language, content
            )
        };

        self.make_gpt_request(&prompt, summary_length).await
    }

    async fn make_gpt_request(&self, prompt: &str, max_tokens: u32) -> Result<String> {
        let response = ureq::post("https://api.openai.com/v1/chat/completions")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(json!({
                "model": MODEL,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "max_tokens": max_tokens,
                "temperature": 0.7
            }))?
            .into_json::<ChatCompletion>()?;

        Ok(response.choices[0].message.content.clone())
    }

    pub async fn collect_for_batch(&self, path: &Path) -> Result<()> {
        if self.should_ignore(path) {
            return Ok(());
        }

        let file_info = utils::get_file_info(path)?;
        if !file_info.is_text || file_info.size > MAX_FILE_SIZE {
            return Ok(());
        }

        if let Ok(content) = fs::read_to_string(path) {
            if !content.trim().is_empty() {
                let mut contents = self.collected_contents.lock().unwrap();
                contents.push((
                    path.to_string_lossy().into_owned(),
                    content[..content.len().min(2000)].to_string(),
                ));
            }
        }

        Ok(())
    }

    pub async fn summarize_batch(&self, custom_query: Option<&str>) -> Result<BatchResult> {
        let contents = self.collected_contents.lock().unwrap();
        
        if contents.is_empty() {
            return Ok(BatchResult::Summaries(HashMap::new()));
        }

        let combined_content = contents
            .iter()
            .map(|(path, content)| format!("File: {}
Content:
{}

", path, content))
            .collect::<String>();

        if let Some(query) = custom_query {
            // For custom queries, return a direct answer
            let response = self.get_gpt_summary(&combined_content, Some(query), 500).await?;
            Ok(BatchResult::Answer(response))
        } else {
            // For regular batch summaries, return a map of file summaries
            let mut summaries = HashMap::new();
            for (path, _) in contents.iter() {
                if let Some(summary) = self.summarize_file(Path::new(path), None).await? {
                    summaries.insert(path.clone(), summary);
                }
            }
            Ok(BatchResult::Summaries(summaries))
        }
    }

    fn should_ignore(&self, path: &Path) -> bool {
        // Check gitignore rules first
        if let Some(ref gitignore) = self.gitignore {
            if gitignore.matched(path, false).is_ignore() {
                return true;
            }
        }

        // Then check custom ignore patterns
        if let Some(ref patterns) = self.custom_patterns {
            let path_str = path.to_string_lossy();
            for pattern in patterns {
                if glob::Pattern::new(pattern).ok()
                    .map(|pat| pat.matches(&path_str))
                    .unwrap_or(false)
                {
                    return true;
                }
            }
        }

        false
    }
}