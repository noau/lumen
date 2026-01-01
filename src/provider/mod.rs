use genai::adapter::AdapterKind;
use genai::chat::{ChatMessage, ChatRequest};
use genai::resolver::{AuthData, Endpoint, ServiceTargetResolver};
use genai::{Client, ClientBuilder, ModelIden, ServiceTarget};
use thiserror::Error;

use crate::ai_prompt::{AIPrompt, AIPromptError};
use crate::command::{draft::DraftCommand, explain::ExplainCommand, operate::OperateCommand};
use crate::config::ProviderInfo;
use crate::config::cli::ProviderType;
use crate::error::LumenError;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("AI request failed: {0}")]
    GenAIError(#[from] genai::Error),

    #[error("API request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("No completion content in response")]
    NoCompletionChoice,

    #[error(transparent)]
    AIPromptError(#[from] AIPromptError),
}

enum ProviderBackend {
    GenAI { client: Client, model: String },
}

pub struct LumenProvider {
    backend: ProviderBackend,
    provider_name: String,
}

/// Provider configuration for custom endpoint providers (OpenRouter, Vercel)
struct CustomProviderConfig {
    endpoint: &'static str,
    env_key: &'static str,
    adapter_kind: AdapterKind,
}

impl LumenProvider {
    pub fn new(
        provider_type: ProviderType,
        api_key: Option<String>,
        model: Option<String>,
    ) -> Result<Self, LumenError> {
        log::trace!("Initializing provider: {:?}", provider_type);
        let (backend, provider_name) = match provider_type {
            // Custom endpoint providers (OpenRouter, Vercel) - use ServiceTargetResolver
            ProviderType::Openrouter | ProviderType::Vercel => {
                let defaults = ProviderInfo::for_provider(provider_type);
                log::trace!("Using custom provider configuration for {}", defaults.display_name);
                let config = match provider_type {
                    ProviderType::Openrouter => CustomProviderConfig {
                        endpoint: "https://openrouter.ai/api/v1/",
                        env_key: defaults.env_key,
                        adapter_kind: AdapterKind::OpenAI,
                    },
                    ProviderType::Vercel => CustomProviderConfig {
                        // Trailing slash is required for URL joining to work correctly
                        endpoint: "https://ai-gateway.vercel.sh/v1/",
                        env_key: defaults.env_key,
                        adapter_kind: AdapterKind::OpenAI,
                    },
                    _ => unreachable!(),
                };

                let model = model.unwrap_or_else(|| defaults.default_model.to_string());
                let model_for_resolver = model.clone();
                log::trace!("Model: {}", model);

                // Get API key from CLI/config or environment
                let auth_env_key = config.env_key;
                if let Some(key) = api_key {
                    log::trace!("Setting API key in environment variable: {}", auth_env_key);
                    // TODO: Audit that the environment access only happens in single-threaded code.
                    unsafe { std::env::set_var(auth_env_key, key) };
                }

                let endpoint = config.endpoint;
                let adapter_kind = config.adapter_kind;

                let target_resolver = ServiceTargetResolver::from_resolver_fn(
                    move |service_target: ServiceTarget| -> Result<ServiceTarget, genai::resolver::Error> {
                        let ServiceTarget { model, .. } = service_target;
                        Ok(ServiceTarget {
                            endpoint: Endpoint::from_static(endpoint),
                            auth: AuthData::from_env(auth_env_key),
                            model: ModelIden::new(adapter_kind, model.model_name),
                        })
                    },
                );

                let client = ClientBuilder::default()
                    .with_service_target_resolver(target_resolver)
                    .build();

                (
                    ProviderBackend::GenAI {
                        client,
                        model: model_for_resolver,
                    },
                    defaults.display_name.to_string(),
                )
            }
            // Native genai providers
            _ => {
                let defaults = ProviderInfo::for_provider(provider_type);

                let model = model.unwrap_or_else(|| defaults.default_model.to_string());
                log::trace!("Model: {}", model);

                // If api_key provided via CLI/config, set it in env so genai picks it up
                if let Some(key) = api_key {
                    if !defaults.env_key.is_empty() {
                        log::trace!("Setting API key in environment variable: {}", defaults.env_key);
                        // TODO: Audit that the environment access only happens in single-threaded code.
                        unsafe { std::env::set_var(defaults.env_key, key) };
                    }
                }

                (
                    ProviderBackend::GenAI {
                        client: Client::default(),
                        model,
                    },
                    defaults.display_name.to_string(),
                )
            }
        };

        Ok(Self {
            backend,
            provider_name,
        })
    }

    async fn complete(&self, prompt: AIPrompt) -> Result<String, ProviderError> {
        match &self.backend {
            ProviderBackend::GenAI { client, model } => {
                log::trace!("Requesting completion from model: {}", model);
                let chat_req = ChatRequest::new(vec![
                    ChatMessage::system(prompt.system_prompt),
                    ChatMessage::user(prompt.user_prompt),
                ]);

                let response = client.exec_chat(model, chat_req, None).await.map_err(|e| {
                    log::error!("AI request failed: {}", e);
                    ProviderError::from(e)
                })?;

                response
                    .first_text()
                    .map(|s| {
                        log::trace!("Received response of length: {}", s.len());
                        s.to_string()
                    })
                    .ok_or_else(|| {
                        log::error!("No completion content in response");
                        ProviderError::NoCompletionChoice
                    })
            }
        }
    }

    pub async fn explain(&self, command: &ExplainCommand) -> Result<String, ProviderError> {
        log::trace!("Building explain prompt");
        let prompt = AIPrompt::build_explain_prompt(command).map_err(|e| {
            log::error!("Failed to build explain prompt: {}", e);
            ProviderError::from(e)
        })?;
        self.complete(prompt).await
    }

    pub async fn draft(&self, command: &DraftCommand) -> Result<String, ProviderError> {
        log::trace!("Building draft prompt");
        let prompt = AIPrompt::build_draft_prompt(command).map_err(|e| {
            log::error!("Failed to build draft prompt: {}", e);
            ProviderError::from(e)
        })?;
        self.complete(prompt).await
    }

    pub async fn operate(&self, command: &OperateCommand) -> Result<String, ProviderError> {
        log::trace!("Building operate prompt for query: {}", command.query);
        let prompt = AIPrompt::build_operate_prompt(command.query.as_str()).map_err(|e| {
            log::error!("Failed to build operate prompt: {}", e);
            ProviderError::from(e)
        })?;
        self.complete(prompt).await
    }

    fn get_model(&self) -> String {
        match &self.backend {
            ProviderBackend::GenAI { model, .. } => model.clone(),
        }
    }
}

impl std::fmt::Display for LumenProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.provider_name, self.get_model())
    }
}
