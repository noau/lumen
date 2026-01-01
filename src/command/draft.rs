use std::io::{IsTerminal, Write};

use crate::{
    config::configuration::DraftConfig, error::LumenError, git_entity::GitEntity,
    provider::LumenProvider,
};

pub struct DraftCommand {
    pub git_entity: GitEntity,
    pub context: Option<String>,
    pub draft_config: DraftConfig,
    pub include_diff: bool,
}

impl DraftCommand {
    pub async fn execute(&self, provider: &LumenProvider) -> Result<(), LumenError> {
        log::trace!("Executing DraftCommand");
        let result = provider.draft(self).await?;
        log::trace!("Draft generated successfully, length: {}", result.len());

        // Only add newline when outputting to terminal, not when piped (e.g., `lumen draft | pbcopy`)
        if std::io::stdout().is_terminal() {
            log::trace!("Outputting to terminal");
            println!("{result}");
        } else {
            log::trace!("Outputting to pipe/file");
            print!("{result}");
        }
        std::io::stdout().flush()?;
        Ok(())
    }
}
