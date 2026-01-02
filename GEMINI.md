# GEMINI.md - Lumen Project Context

## Project Overview
Lumen is an AI-powered command-line tool designed to enhance Git workflows. It leverages various AI providers (primarily Gemini, but also OpenAI, Claude, Groq, and others) to generate commit messages, summarize diffs, explain past commits, and convert natural language queries into Git commands.

### Core Technologies
- **Language:** Rust (Edition 2024)
- **CLI Framework:** `clap` (with derive and env features)
- **Async Runtime:** `tokio`
- **AI Integration:** `genai` crate (multi-provider support)
- **Git Interaction:** Standard `git` CLI (via process execution)
- **UI/UX:** `inquire` for interactive prompts, `fzf` for fuzzy finding, and `mdcat` for terminal markdown rendering.

### Architecture
- `src/main.rs`: Entry point and command dispatcher.
- `src/command/`: Subcommand implementations (`Explain`, `List`, `Draft`, `Operate`, `Configure`).
- `src/config/`: Configuration management, including provider and API key setup.
- `src/git_entity/`: Abstractions for Git concepts like Commits and Diffs.
- `src/provider/`: AI provider abstraction using the `genai` library.
- `src/ai_prompt.rs`: System and user prompt definitions for AI interactions.

## Building and Running

### Prerequisites
- Rust toolchain (latest stable)
- `git` installed and in PATH
- (Optional) `fzf` for the `list` command
- (Optional) `mdcat` for pretty markdown rendering

### Commands
- **Build:** `cargo build`
- **Run:** `cargo run -- <command>`
- **Test:** `cargo test`
- **Install:** `cargo install --path .`

### Key CLI Subcommands
- `lumen configure`: Interactively set up your AI provider and API key.
- `lumen draft`: Generate a commit message for staged changes.
- `lumen explain`: Summarize changes in a specific commit or range (e.g., `lumen explain HEAD~1`).
- `lumen list`: Interactively select a commit from history using `fzf` and explain its changes.
- `lumen operate "<query>"`: Convert a natural language instruction into a Git command.

## Development Conventions

### Code Style
- Follows standard Rust idioms and formatting (`rustfmt`).
- Uses `thiserror` for error handling and `log` with `env_logger` for tracing.
- Prompts are managed in `src/ai_prompt.rs` using `indoc` and `formatdoc` for readability.

### Testing
- Unit tests are located within the modules they test (e.g., `src/commit_reference.rs`, `src/git_entity/commit.rs`).
- Run tests with `cargo test`.

### AI Providers
Lumen supports a wide range of providers via the `genai` crate:
- `gemini` (Default/Preferred)
- `openai`, `claude`, `groq`, `ollama`, `openrouter`, `deepseek`, `xai`, `vercel`.

### Logging
- Use `--verbose` or `-v` for trace-level logging.
- Set `RUST_LOG` environment variable for more granular control.
- Logs can be redirected to a file using `--log-target <path>`.
