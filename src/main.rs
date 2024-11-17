use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;

mod file_explorer;
mod gpt_client;
mod utils;

use file_explorer::FileExplorer;
use gpt_client::GPTClient;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory path to explore (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Enable GPT-powered file summarization (optimized for SQL files)
    #[arg(long)]
    ai: bool,

    /// Custom query for GPT summarization
    #[arg(long)]
    ai_query: Option<String>,

    /// Summarize all files in a single batch with optional custom query
    #[arg(long)]
    ai_whole: Option<String>,

    /// Maximum directory depth to explore
    #[arg(long, default_value_t = 3)]
    max_depth: u32,

    /// Length of the summary (short: ~50 words, medium: ~100 words, long: ~200 words)
    #[arg(long, default_value = "medium")]
    summary_length: String,

    /// Language for the summary (e.g., "english", "spanish", "french", etc.)
    #[arg(long, default_value = "english")]
    language: String,

    /// Force update cache entries
    #[arg(long)]
    update: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let mut explorer = FileExplorer::new(args.max_depth);

    if args.ai || args.ai_query.is_some() || args.ai_whole.is_some() {
        let client = GPTClient::new(&args.summary_length, &args.language, args.update)?;
        explorer.set_summarizer(client, args.ai_query, args.ai_whole);
    }

    explorer.explore(args.path).await?;

    Ok(())
}