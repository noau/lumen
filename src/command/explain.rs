use spinoff::{Color, Spinner, spinners};

use crate::{
    error::LumenError,
    git_entity::{diff::Diff, GitEntity},
    provider::LumenProvider,
};

use super::LumenCommand;

pub struct ExplainCommand {
    pub git_entity: GitEntity,
    pub query: Option<String>,
}

impl ExplainCommand {
    pub async fn execute(&self, provider: &LumenProvider) -> Result<(), LumenError> {
        LumenCommand::print_with_mdcat(self.git_entity.format_static_details(provider))?;
        if let Some(query) = &self.query {
            LumenCommand::print_with_mdcat(format!("`query`: {query}"))?;
        }

        let spinner_text = match &self.query {
            Some(_) => "Generating answer...".to_string(),
            None => "Generating summary...".to_string(),
        };

        let mut spinner = Spinner::new(spinners::Dots, spinner_text, Color::Blue);
        let result = provider.explain(self).await?;

        // Cache the result if it's a working tree diff
        if let GitEntity::Diff(Diff::WorkingTree { diff, .. }) = &self.git_entity {
            // We ignore cache errors to not break the flow
            let _ = crate::cache::save_explanation(diff, &result);
        }

        spinner.success("Done");

        LumenCommand::print_with_mdcat(result)?;
        Ok(())
    }
}
