use crate::config::{ALL_PROVIDERS, LumenConfig, ProviderInfo};
use crate::error::LumenError;
use dirs::home_dir;
use inquire::{Select, Text};
use serde_json::{Value, json};
use std::fmt;
use std::fs;

/// Wrapper for display in the selection prompt
struct ProviderChoice(&'static ProviderInfo);

impl fmt::Display for ProviderChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display_name)
    }
}

/// Command to handle interactive configuration of Lumen features.
pub struct ConfigureCommand;

impl ConfigureCommand {
    /// Executes the configuration command.
    ///
    /// If `show` is true, prints the current configuration.
    /// Otherwise, starts the interactive configuration wizard.
    pub fn execute(config: LumenConfig, show: bool) -> Result<(), LumenError> {
        if show {
            log::trace!("Showing current configuration");
            Self::show_config(config);
            return Ok(());
        }

        log::trace!("Executing ConfigureCommand interactive wizard");
        println!("\n  \x1b[1;36mLumen Configuration\x1b[0m\n");

        let provider = Self::select_provider()?;
        log::trace!("Selected provider: {}", provider.display_name);
        let api_key = Self::get_api_key(provider)?;
        let model = Self::get_model_name(provider)?;

        Self::save_config(provider, api_key.as_deref(), model.as_deref())?;

        let config_path = Self::get_config_path()?;
        println!(
            "\n  \x1b[1;32m✓\x1b[0m Configuration saved to \x1b[2m{}\x1b[0m\n",
            config_path.join("lumen.config.json").display()
        );

        Ok(())
    }

    /// Displays the current configuration with a masked API key.
    fn show_config(config: LumenConfig) {
        println!("\n  \x1b[1;36mCurrent Lumen Configuration\x1b[0m\n");
        println!("  Provider:            {:?}", config.provider);
        println!(
            "  Model:               {}",
            config.model.as_deref().unwrap_or("default")
        );

        let masked_key = config
            .api_key
            .map(|key| {
                let prefix_len = std::cmp::min(key.len() - 3, 5);
                format!("{}*****", &key[..prefix_len])
            })
            .unwrap_or_else(|| "not set".to_string());

        println!("  API Key:             {}", masked_key);
        println!("  Draft Commit Types:  {}", config.draft.commit_types);
        println!();
    }

    /// Prompts the user to select an AI provider from the supported list.
    fn select_provider() -> Result<&'static ProviderInfo, LumenError> {
        let options: Vec<ProviderChoice> = ALL_PROVIDERS.iter().map(ProviderChoice).collect();

        let selection = Select::new("Select your default AI provider:", options)
            .with_help_message("↑↓ to move, enter to select, type to filter")
            .prompt()
            .map_err(|e| {
                log::error!("Provider selection failed: {}", e);
                LumenError::ConfigurationError(e.to_string())
            })?;

        Ok(selection.0)
    }

    /// Prompts the user for an API key if the provider requires one.
    /// Returns `None` if the user leaves the input empty (to use env var) or if the provider
    /// is local (e.g. Ollama).
    fn get_api_key(provider: &ProviderInfo) -> Result<Option<String>, LumenError> {
        if provider.env_key.is_empty() {
            log::trace!(
                "Provider {} does not require an API key",
                provider.display_name
            );
            println!("\n  \x1b[2mOllama runs locally — no API key needed.\x1b[0m");
            return Ok(None);
        }

        let prompt = format!(
            "Enter your API key (or leave empty to use {}):",
            provider.env_key
        );

        let api_key = Text::new(&prompt).prompt().map_err(|e| {
            log::error!("API key input failed: {}", e);
            LumenError::ConfigurationError(e.to_string())
        })?;

        if api_key.is_empty() {
            log::trace!("User left API key empty, will use environment variable");
            Ok(None)
        } else {
            log::trace!("User provided an API key");
            Ok(Some(api_key))
        }
    }

    /// Prompts the user for a custom model name.
    /// Returns `None` if the user accepts the default model by pressing Enter.
    fn get_model_name(provider: &ProviderInfo) -> Result<Option<String>, LumenError> {
        let prompt = format!(
            "Enter model name (leave empty for default: {}):",
            provider.default_model
        );

        let model = Text::new(&prompt)
            .with_help_message("Press Enter to use the default model")
            .prompt()
            .map_err(|e| {
                log::error!("Model name input failed: {}", e);
                LumenError::ConfigurationError(e.to_string())
            })?;

        if model.is_empty() {
            log::trace!("Using default model: {}", provider.default_model);
            Ok(None)
        } else {
            log::trace!("User specified custom model: {}", model);
            Ok(Some(model))
        }
    }

    /// Resolves the path to the configuration directory (`~/.config/lumen`).
    fn get_config_path() -> Result<std::path::PathBuf, LumenError> {
        let path = home_dir().ok_or_else(|| {
            log::error!("Could not determine home directory");
            LumenError::ConfigurationError("Could not determine home directory".to_string())
        })?;
        let mut path = path;
        path.push(".config");
        path.push("lumen");
        Ok(path)
    }

    /// Saves the selected configuration to the JSON config file.
    /// If `model` is `None`, any existing `model` key in the config is removed to ensure
    /// the provider's default is used.
    fn save_config(
        provider: &ProviderInfo,
        api_key: Option<&str>,
        model: Option<&str>,
    ) -> Result<(), LumenError> {
        let config_dir = Self::get_config_path()?;
        log::trace!("Creating config directory: {:?}", config_dir);
        fs::create_dir_all(&config_dir)?;

        let config_file = config_dir.join("lumen.config.json");

        let mut config: Value = if config_file.exists() {
            log::trace!("Updating existing config file: {:?}", config_file);
            let content = fs::read_to_string(&config_file)?;
            serde_json::from_str(&content).unwrap_or_else(|e| {
                log::warn!("Failed to parse existing config, starting fresh: {}", e);
                json!({})
            })
        } else {
            log::trace!("Creating new config file: {:?}", config_file);
            json!({})
        };

        // Get provider ID from the type
        config["provider"] = json!(provider.id);

        if let Some(key) = api_key {
            config["api_key"] = json!(key);
        }

        if let Some(m) = model {
            config["model"] = json!(m);
        } else {
            // Remove model key to use provider default
            config.as_object_mut().map(|obj| obj.remove("model"));
        }

        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&config_file, content)?;
        log::trace!("Configuration written successfully");

        Ok(())
    }
}
