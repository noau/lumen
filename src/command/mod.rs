use std::io::Write;
use draft::DraftCommand;
use explain::ExplainCommand;
use list::ListCommand;
use operate::OperateCommand;
use std::process::Stdio;

use crate::config::configuration::DraftConfig;
use crate::error::LumenError;
use crate::git_entity::GitEntity;
use crate::git_entity::diff::Diff;
use crate::provider::LumenProvider;

pub mod configure;
pub mod draft;
pub mod explain;
pub mod list;
pub mod operate;

#[derive(Debug)]
pub enum CommandType {
    Explain {
        git_entity: GitEntity,
        query: Option<String>,
    },
    List,
    Draft(Option<String>, DraftConfig),
    Operate {
        query: String,
    },
}

pub struct LumenCommand {
    provider: LumenProvider,
    no_mdcat: bool,
}

impl LumenCommand {
    pub fn new(provider: LumenProvider, no_mdcat: bool) -> Self {
        LumenCommand { provider, no_mdcat }
    }

    pub async fn execute(&self, command_type: CommandType) -> Result<(), LumenError> {
        match command_type {
            CommandType::Explain { git_entity, query } => {
                log::trace!("Dispatching Explain command");
                ExplainCommand {
                    git_entity,
                    query,
                    no_mdcat: self.no_mdcat,
                }
                .execute(&self.provider)
                .await
            }
            CommandType::List => {
                log::trace!("Dispatching List command");
                ListCommand {
                    no_mdcat: self.no_mdcat,
                }
                .execute(&self.provider)
                .await
            }
            CommandType::Draft(context, draft_config) => {
                log::trace!("Dispatching Draft command");
                let diff = Diff::from_working_tree(true)?;
                let mut context = context;
                let mut include_diff = true;

                // Check for cached explanation
                if let Diff::WorkingTree {
                    diff: ref diff_content,
                    ..
                } = diff
                {
                    if let Some(summary) = crate::cache::get_explanation(diff_content) {
                        log::trace!("Found cached explanation for draft context");
                        let cached_context =
                            format!("Previous explanation of these changes:\n{}", summary);
                        context = match context {
                            Some(c) => Some(format!("{}\n\n{}", c, cached_context)),
                            None => Some(cached_context),
                        };
                        include_diff = false;
                    }
                }

                DraftCommand {
                    git_entity: GitEntity::Diff(diff),
                    draft_config,
                    context,
                    include_diff,
                }
                .execute(&self.provider)
                .await
            }
            CommandType::Operate { query } => {
                log::trace!("Dispatching Operate command");
                OperateCommand {
                    query,
                    no_mdcat: self.no_mdcat,
                }
                .execute(&self.provider)
                .await
            }
        }
    }

    fn get_sha_from_fzf() -> Result<String, LumenError> {
        let command = "git log --color=always --format='%C(auto)%h%d %s %C(black)%C(bold)%cr' | fzf --ansi --reverse --bind='enter:become(echo {1})'";

        log::trace!("Executing fzf command: {}", command);
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        if !output.status.success() {
            let mut stderr = String::from_utf8(output.stderr)?;
            stderr.pop();
            log::error!("fzf command failed: {}", stderr);

            let hint = match &stderr {
                stderr if stderr.contains("fzf: command not found") => {
                    Some("`list` command requires fzf")
                }
                _ => None,
            };

            let hint = match hint {
                Some(hint) => format!("(hint: {})", hint),
                None => String::new(),
            };

            return Err(LumenError::CommandError(format!("{} {}", stderr, hint)));
        }

        let mut sha = String::from_utf8(output.stdout)?;
        sha.pop(); // remove trailing newline from echo
        log::trace!("Selected SHA from fzf: {}", sha);

        Ok(sha)
    }

    pub fn print_with_mdcat(content: String, no_mdcat: bool) -> Result<(), LumenError> {
        if no_mdcat {
            log::trace!("mdcat disabled by flag, printing raw content");
            println!("{}", content);
            return Ok(());
        }

        let try_mdcat = |text: &str| -> Result<(), Box<dyn std::error::Error>> {
            log::trace!("Attempting to render with mdcat");
            let mut child = std::process::Command::new("mdcat")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(text.as_bytes())?;
            }

            let output = child.wait_with_output()?;
            if output.status.success() {
                log::trace!("mdcat rendered successfully");
                println!("{}", String::from_utf8(output.stdout)?);
                Ok(())
            } else {
                log::warn!("mdcat failed with status: {:?}", output.status);
                Err("mdcat failed".into())
            }
        };

        if try_mdcat(&content).is_err() {
            log::trace!("Falling back to raw println");
            println!("{}", content);
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn execute_bash_command(command: &str) -> Result<(), LumenError> {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()?;

        if !output.status.success() {
            let mut stderr = String::from_utf8(output.stderr)?;
            stderr.pop();
            return Err(LumenError::CommandError(stderr));
        }
        println!("{}", String::from_utf8(output.stdout)?);
        Ok(())
    }

    #[allow(dead_code)]
    fn execute_bash_command_with_confirmation(command: &str) -> Result<(), LumenError> {
        let mut input = String::new();
        println!("{} (y/N)", command);
        std::io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            return Err(LumenError::CommandError("Aborted".to_string()));
        }
        LumenCommand::execute_bash_command(command)
    }
}
