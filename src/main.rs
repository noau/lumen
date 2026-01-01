use clap::Parser;
use commit_reference::CommitReference;
use config::LumenConfig;
use config::cli::{Cli, Commands};
use env_logger::{Builder, Target};
use error::LumenError;
use git_entity::{GitEntity, commit::Commit, diff::Diff};
use log::LevelFilter;
use std::io::Read;
use std::process;

mod ai_prompt;
mod cache;
mod command;
mod commit_reference;
mod config;
mod error;
mod git_entity;
mod provider;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("\x1b[91m\rerror:\x1b[0m {e}");
        process::exit(1);
    }
}

async fn run() -> Result<(), LumenError> {
    let cli = Cli::parse();

    let mut builder = Builder::from_default_env();
    if let Some(log_path) = &cli.log_target {
        let target = std::fs::File::create(log_path)?;
        builder.target(Target::Pipe(Box::new(target)));
    } else {
        builder.target(Target::Stdout);
    }

    if cli.verbose {
        builder.filter(None, LevelFilter::Trace);
    } else if std::env::var("RUST_LOG").is_err() {
        builder.filter(None, LevelFilter::Warn);
    }

    builder.init();
    log::trace!("Logger initialized");

    let config = match LumenConfig::build(&cli) {
        Ok(config) => config,
        Err(e) => return Err(e),
    };
    log::trace!("Configuration loaded");

    let provider = provider::LumenProvider::new(
        config.provider,
        config.api_key.clone(),
        config.model.clone(),
    )?;
    log::trace!("Provider initialized");

    let command = command::LumenCommand::new(provider, cli.no_mdcat);

    match cli.command {
        Commands::Explain {
            reference,
            staged,
            query,
        } => {
            log::trace!("Executing Explain command");
            let git_entity = match reference {
                Some(CommitReference::Single(input)) => {
                    let sha = if input == "-" {
                        read_from_stdin()?
                    } else {
                        input
                    };
                    GitEntity::Commit(Commit::new(sha)?)
                }
                Some(CommitReference::Range { from, to }) => {
                    GitEntity::Diff(Diff::from_commits_range(&from, &to, false)?)
                }
                Some(CommitReference::TripleDots { from, to }) => {
                    GitEntity::Diff(Diff::from_commits_range(&from, &to, true)?)
                }
                None => {
                    // Default: show uncommitted diff
                    GitEntity::Diff(Diff::from_working_tree(staged)?)
                }
            };

            command
                .execute(command::CommandType::Explain { git_entity, query })
                .await?;
        }
        Commands::List => {
            log::trace!("Executing List command");
            command.execute(command::CommandType::List).await?
        }
        Commands::Draft { context } => {
            log::trace!("Executing Draft command");
            command
                .execute(command::CommandType::Draft(context, config.draft))
                .await?
        }
        Commands::Operate { query } => {
            log::trace!("Executing Operate command");
            command
                .execute(command::CommandType::Operate { query })
                .await?;
        }
        Commands::Configure { show } => {
            log::trace!("Executing Configure command (show: {})", show);
            command::configure::ConfigureCommand::execute(config, show)?;
        }
    }

    Ok(())
}

fn read_from_stdin() -> Result<String, LumenError> {
    let mut buffer = String::new();
    std::io::stdin().read_to_string(&mut buffer)?;

    eprintln!("Reading commit SHA from stdin: '{}'", buffer.trim());
    Ok(buffer)
}
