use crate::{
    error::LumenError,
    git_entity::{GitEntity, commit::Commit},
    provider::LumenProvider,
};

use super::{LumenCommand, explain::ExplainCommand};

pub struct ListCommand {
    pub no_mdcat: bool,
}

impl ListCommand {
    pub async fn execute(&self, provider: &LumenProvider) -> Result<(), LumenError> {
        log::trace!("Executing ListCommand");
        let sha = LumenCommand::get_sha_from_fzf()?;
        log::trace!("Selected SHA from fzf: {}", sha);
        let git_entity = GitEntity::Commit(Commit::new(sha)?);
        ExplainCommand {
            git_entity,
            query: None,
            no_mdcat: self.no_mdcat,
        }
        .execute(provider)
        .await
    }
}
