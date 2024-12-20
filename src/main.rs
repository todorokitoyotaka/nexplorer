use clap::Parser;
use std::path::PathBuf;
use anyhow::Result;
use env_logger::Env;
use log::info;

use nexplorer::file_explorer::FileExplorer;
use nexplorer::gpt_client::GPTClient;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File or directory paths to explore
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// Enable GPT-powered file summarization
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

    /// Length of the summary (smart: automatic based on file size, short: ~50 words, medium: ~100 words, long: ~200 words, super: ~500 words, or a custom number)
    #[arg(long, default_value = "medium")]
    summary_length: String,

    /// Language for the summary (e.g., "english", "japanese", etc.)
    #[arg(long, default_value = "english")]
    language: String,

    /// Force update cache entries
    #[arg(long)]
    update: bool,

    /// Custom ignore patterns (comma-separated)
    #[arg(long)]
    ignore: Option<String>,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logger with appropriate level based on debug flag and release mode
    let env = Env::default()
        .filter_or("RUST_LOG", if args.debug {
            "debug"
        } else if cfg!(debug_assertions) {
            "info"
        } else {
            "warn"
        });
    env_logger::init_from_env(env);

    info!("Starting nexplorer with {} path(s)", args.paths.len());
    
    let mut explorer = FileExplorer::new(args.max_depth);

    if args.ai || args.ai_query.is_some() || args.ai_whole.is_some() {
        let client = GPTClient::new(&args.summary_length, &args.language, args.update, args.ignore.as_deref())?;
        explorer.set_summarizer(client, args.ai_query, args.ai_whole);
    }

    // Process each path provided
    for path in args.paths {
        explorer.explore(path).await?;
    }

    Ok(())
}
