use std::path::Path;
use anyhow::Result;
use walkdir::WalkDir;
use crate::gpt_client::{GPTClient, BatchResult};
use crate::utils::{format_size, get_file_info};

pub struct FileExplorer {
    max_depth: u32,
    summarizer: Option<GPTClient>,
    custom_query: Option<String>,
    batch_mode: bool,
    batch_query: Option<String>,
    total_files: u32,
    total_dirs: u32,
}

impl FileExplorer {
    pub fn new(max_depth: u32) -> Self {
        Self {
            max_depth,
            summarizer: None,
            custom_query: None,
            batch_mode: false,
            batch_query: None,
            total_files: 0,
            total_dirs: 0,
        }
    }

    pub fn set_summarizer(&mut self, client: GPTClient, custom_query: Option<String>, batch_query: Option<String>) {
        self.summarizer = Some(client);
        self.custom_query = custom_query;
        self.batch_mode = batch_query.is_some();
        self.batch_query = batch_query;
    }

    pub async fn explore<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        println!("\nExploring: {}", path.display());
        println!("{}", "=".repeat(80));

        if path.is_file() {
            self.process_file(path, 0).await?;
        } else {
            self.explore_directory(path).await?;
        }

        if self.batch_mode {
            if let Some(summarizer) = &self.summarizer {
                // Only show "Generating batch summaries..." for regular summaries
                if self.batch_query.is_none() {
                    println!("\nGenerating batch summaries...");
                }

                match summarizer.summarize_batch(self.batch_query.as_deref()).await {
                    Ok(BatchResult::Summaries(summaries)) => {
                        if !summaries.is_empty() {
                            println!("\nFile Summaries:");
                            println!("{}", "=".repeat(80));
                            for (file_name, summary) in summaries {
                                println!("\n📄 {}:", file_name);
                                println!("   📝 {}", summary);
                            }
                        }
                    }
                    Ok(BatchResult::Answer(answer)) => {
                        // For custom queries, display the direct answer
                        println!("\n📝 {}", answer);
                    }
                    Err(e) => eprintln!("\n⚠️ Error processing files: {}", e),
                }
            }
        }

        println!("\nSummary:");
        println!("Total directories: {}", self.total_dirs);
        println!("Total files: {}", self.total_files);

        Ok(())
    }

    async fn explore_directory(&mut self, path: &Path) -> Result<()> {
        for entry in WalkDir::new(path)
            .max_depth(self.max_depth as usize)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let depth = entry.depth();
            let indent = "  ".repeat(depth);

            if entry.file_type().is_dir() {
                self.total_dirs += 1;
                println!("{}📁 {}/", indent, entry.file_name().to_string_lossy());
            } else {
                self.process_file(entry.path(), depth).await?;
            }
        }
        Ok(())
    }

    async fn process_file(&mut self, path: &Path, depth: usize) -> Result<()> {
        self.total_files += 1;
        let indent = "  ".repeat(depth);
        let file_info = get_file_info(path)?;

        println!("{}📄 {} ({})", 
            indent,
            path.file_name().unwrap_or_default().to_string_lossy(),
            format_size(file_info.size)
        );

        if let Some(summarizer) = &self.summarizer {
            if file_info.is_text {
                if self.batch_mode {
                    summarizer.collect_for_batch(path).await?;
                } else {
                    match summarizer.summarize_file(path, self.custom_query.as_deref()).await {
                        Ok(Some(summary)) => {
                            println!("{}   📝 Summary: {}", indent, summary);
                        }
                        Ok(None) => {}
                        Err(e) => {
                            eprintln!("{}   ⚠️ Failed to generate summary: {}", indent, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
