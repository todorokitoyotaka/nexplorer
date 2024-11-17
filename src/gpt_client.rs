use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Mutex;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_FILE_SIZE: u64 = 1024 * 1024; // 1MB
const MODEL: &str = "gpt-4o-mini";
const CACHE_DIR: &str = ".cache";

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
}

impl GPTClient {
    pub fn new(summary_length: &str, language: &str, force_update: bool) -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .context("OPENAI_API_KEY environment variable is not set")?;
            
        let max_tokens = match summary_length {
            "short" => 50,
            "long" => 200,
            _ => 100, // medium or any other value
        };

        // Create cache directory if it doesn't exist
        let cache_dir = PathBuf::from(CACHE_DIR);
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        Ok(Self {
            api_key,
            collected_contents: Mutex::new(Vec::new()),
            cache_dir,
            max_tokens,
            summary_length: summary_length.to_string(),
            language: language.to_string(),
            force_update,
        })
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
        if !self.is_supported_file(path) {
            return Ok(None);
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_FILE_SIZE {
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

        // Generate new summary if not in cache
        let summary = if let Some(query) = custom_query {
            self.get_gpt_summary(&content, Some(query)).await?
        } else if self.is_sql_file(path) {
            self.get_sql_summary(&content).await?
        } else {
            self.get_gpt_summary(&content, None).await?
        };

        // Add to cache
        self.add_to_cache(content_hash, summary.clone())?;
        Ok(Some(summary))
    }

    pub async fn collect_for_batch(&self, path: &Path) -> Result<()> {
        if !self.is_supported_file(path) {
            return Ok(());
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_FILE_SIZE {
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

        // For custom queries, combine all contents and return a single answer
        if let Some(query) = custom_query {
            let combined_content = contents
                .iter()
                .map(|(path, content)| format!("File: {}\nContent:\n{}", path, content))
                .collect::<Vec<_>>()
                .join("\n\n===FILE SEPARATOR===\n\n");

            let prompt = format!(
                "{} (respond in {})\n\nAnalyze the following files to answer the question:\n\n{}",
                query, self.language, combined_content
            );

            let response = self.make_gpt_request(&prompt).await?;
            return Ok(BatchResult::Answer(response));
        }

        // For regular summaries, keep the existing logic
        let mut summaries = HashMap::new();
        let mut uncached_contents = Vec::new();

        for (path, content) in contents.iter() {
            let content_hash = self.calculate_content_hash(content, custom_query);
            if let Some(cached_summary) = self.get_from_cache(&content_hash) {
                summaries.insert(path.clone(), cached_summary);
            } else {
                uncached_contents.push((path.clone(), content.clone(), content_hash));
            }
        }

        if !uncached_contents.is_empty() {
            let files_content = uncached_contents
                .iter()
                .map(|(path, content, _)| format!("File: {}\nContent:\n{}", path, content))
                .collect::<Vec<_>>()
                .join("\n\n===FILE SEPARATOR===\n\n");

            let prompt = format!(
                "Analyze multiple files and provide a {} summary for each ({} words) in {}. \
                Focus on the main purpose of each file. Format the response as:\n\
                File1: Summary of first file\n\
                File2: Summary of second file\n\
                And so on.\n\n{}",
                self.summary_length, self.max_tokens, self.language, files_content
            );

            let response = self.make_gpt_request(&prompt).await?;
            let batch_summaries = self.parse_batch_summaries(&response)?;

            for (path, _, content_hash) in uncached_contents {
                if let Some(summary) = batch_summaries.get(&path) {
                    self.add_to_cache(content_hash, summary.clone())?;
                    summaries.insert(path, summary.clone());
                }
            }
        }

        Ok(BatchResult::Summaries(summaries))
    }

    async fn get_gpt_summary(&self, content: &str, custom_prompt: Option<&str>) -> Result<String> {
        let prompt = if let Some(query) = custom_prompt {
            format!("{} (respond in {})\n\n{}", query, self.language, &content[..content.len().min(4000)])
        } else {
            format!(
                "Summarize the following file content in a {} summary ({} words) in {}. \
                Focus on its main purpose and key elements:\n\n{}",
                self.summary_length, self.max_tokens, self.language, &content[..content.len().min(4000)]
            )
        };

        self.make_gpt_request(&prompt).await
    }

    async fn get_sql_summary(&self, content: &str) -> Result<String> {
        let prompt = format!(
            "Analyze the following SQL code and provide a {} summary ({} words) in {}. \
            Focus on: table operations (CREATE, ALTER, DROP), main table names, \
            key relationships, and important constraints or indices if present:\n\n{}",
            self.summary_length, self.max_tokens, self.language, &content[..content.len().min(4000)]
        );

        self.make_gpt_request(&prompt).await
    }

    async fn make_gpt_request(&self, prompt: &str) -> Result<String> {
        let response = ureq::post("https://api.openai.com/v1/chat/completions")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .send_json(json!({
                "model": MODEL,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }],
                "max_tokens": self.max_tokens,
                "temperature": 0.7
            }))?
            .into_json::<ChatCompletion>()?;

        Ok(response.choices[0].message.content.clone())
    }

    fn is_supported_file(&self, path: &Path) -> bool {
        let mime_type = mime_guess::from_path(path).first_or_octet_stream();
        mime_type.type_() == "text"
            || mime_type.subtype() == "javascript"
            || mime_type.subtype() == "json"
            || mime_type.subtype() == "xml"
            || mime_type.subtype() == "sql"
            || path.extension().map_or(false, |ext| ext == "sql")
    }

    fn is_sql_file(&self, path: &Path) -> bool {
        path.extension().map_or(false, |ext| ext == "sql")
            || mime_guess::from_path(path)
                .first_or_octet_stream()
                .subtype() == "sql"
    }

    fn parse_batch_summaries(&self, text: &str) -> Result<HashMap<String, String>> {
        let mut summaries = HashMap::new();
        let mut current_file = None;
        let mut current_summary = Vec::new();

        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(pos) = line.find(':') {
                if !line.starts_with(' ') {
                    if let Some(file) = current_file.take() {
                        summaries.insert(file, current_summary.join(" "));
                        current_summary.clear();
                    }
                    current_file = Some(line[..pos].trim().to_string());
                    current_summary.push(line[pos + 1..].trim());
                }
            } else if let Some(_) = current_file {
                current_summary.push(line.trim());
            }
        }

        if let Some(file) = current_file {
            summaries.insert(file, current_summary.join(" "));
        }

        Ok(summaries)
    }
}