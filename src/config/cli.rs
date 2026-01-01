use clap::{Parser, Subcommand, ValueEnum};
use std::str::FromStr;

use crate::commit_reference::CommitReference;

#[derive(Parser)]
#[command(name = "lumen")]
#[command(about = "AI-powered CLI tool for git commit summaries", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Path to configuration file eg: ./path/to/lumen.config.json
    #[arg(long)]
    pub config: Option<String>,

    #[arg(value_enum, short = 'p', long = "provider")]
    pub provider: Option<ProviderType>,

    #[arg(short = 'k', long = "api-key")]
    pub api_key: Option<String>,

    #[arg(short = 'm', long = "model")]
    pub model: Option<String>,

    /// Disable mdcat for markdown rendering
    #[arg(long, global = true)]
    pub no_mdcat: bool,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Log output target (defaults to stdout)
    #[arg(long, global = true)]
    pub log_target: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
pub enum ProviderType {
    Openai,
    Groq,
    Claude,
    Ollama,
    Openrouter,
    Deepseek,
    Gemini,
    Xai,
    Vercel,
}

impl FromStr for ProviderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(ProviderType::Openai),
            "groq" => Ok(ProviderType::Groq),
            "claude" => Ok(ProviderType::Claude),
            "ollama" => Ok(ProviderType::Ollama),
            "openrouter" => Ok(ProviderType::Openrouter),
            "deepseek" => Ok(ProviderType::Deepseek),
            "gemini" => Ok(ProviderType::Gemini),
            "xai" => Ok(ProviderType::Xai),
            "vercel" => Ok(ProviderType::Vercel),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Explain the changes in a commit, or the current diff (default)
    Explain {
        /// Commit reference: SHA, HEAD, HEAD~3..HEAD, main..feature, main...feature
        #[arg(value_parser = clap::value_parser!(CommitReference))]
        reference: Option<CommitReference>,

        /// Use staged diff only (when showing uncommitted changes)
        #[arg(long)]
        staged: bool,

        /// Ask a question instead of summary
        #[arg(short, long)]
        query: Option<String>,
    },
    /// List all commits in an interactive fuzzy-finder, and summarize the changes
    List,
    /// Generate a commit message for the staged changes
    Draft {
        /// Add context to communicate intent
        #[arg(short, long)]
        context: Option<String>,
    },

    Operate {
        #[arg()]
        query: String,
    },
    /// Interactively configure Lumen (provider, API key)
    Configure,
}
