# Lumen Project Context

## Project Overview
**Lumen** is a command-line interface (CLI) tool written in Rust that leverages Artificial Intelligence to streamline Git workflows. It assists developers by generating commit messages, visualizing and explaining diffs, and translating natural language queries into Git commands.

## Architecture & Code Structure
The project follows a standard Rust binary application structure with a modular design:

- **`src/main.rs`**: The entry point of the application. It initializes the configuration, sets up the AI provider, and dispatches the parsed CLI commands.
- **`src/command/`**: Contains the implementation for each CLI command.
    - `mod.rs`: Central dispatcher enum `CommandType` and `LumenCommand` struct.
    - `draft.rs`: Logic for generating commit messages (`lumen draft`).
    - `explain.rs`: Logic for explaining changes (`lumen explain`).
    - `operate.rs`: Logic for natural language git operations (`lumen operate`).
    - `list.rs`: Interactive commit list (`lumen list`).
    - `configure.rs`: Interactive configuration setup (`lumen configure`).
- **`src/config/`**: Handles configuration loading and parsing.
    - `configuration.rs`: Defines the `LumenConfig` struct.
    - `providers.rs`: AI provider configuration details.
- **`src/git_entity/`**: Abstractions for Git objects.
    - `commit.rs`: handling commit data.
    - `diff.rs`: handling diff generation and parsing.
- **`src/provider/`**: Interfaces with various AI providers (OpenAI, Claude, Gemini, etc.) via the `genai` crate.
- **`src/ai_prompt.rs`**: Likely contains the system prompts used to instruct the LLMs.

## Development Workflow

### Prerequisites
- **Rust**: Ensure you have the latest stable version of Rust and Cargo installed (`rustup`).
- **Git**: The tool relies heavily on local `git` commands.
- **Optional Tools**:
    - `fzf`: Required for the `list` command's interactive fuzzy finding.
    - `mdcat`: Required for rendering Markdown output in the terminal.

### Building and Running
Use standard Cargo commands to build and run the project:

```bash
# Build the project
cargo build

# Run the project (pass arguments after --)
cargo run -- --help
cargo run -- draft
cargo run -- explain
```

### Testing
Run the test suite to ensure changes don't break existing functionality:

```bash
cargo test
```

## Configuration
Lumen requires configuration to access AI features. The configuration file `lumen.config.json` is looked up in the following order:
1. CLI arguments.
2. Custom path via `--config`.
3. Project root.
4. Global configuration directory (`~/.config/lumen/` on *nix, `%USERPROFILE%\.config\lumen\` on Windows).

For development, you can create a `lumen.config.json` in the project root:

```json
{
  "provider": "openai",
  "model": "gpt-4o-mini",
  "api_key": "your-api-key-here"
}
```

## Key Dependencies
- **`clap`**: Command-line argument parsing.
- **`tokio`**: Asynchronous runtime.
- **`genai`**: Universal LLM client.
- **`inquire`**: Interactive terminal prompts.
- **`spinoff`**: Terminal spinners for loading states.
- **`serde`**: Serialization and deserialization.
