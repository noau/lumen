# <p align="center"><img src="https://github.com/user-attachments/assets/896f9239-134a-4428-9bb5-50ea59cdb5c3" alt="lumen" /></p>

> [!IMPORTANT]
> This repository is a customized fork of the original [jnsahaj/lumen](https://github.com/jnsahaj/lumen). 
> 
> **Key Modifications:**
> - **Removed TUI Diff Viewer:** The interactive `diff` subcommand and its heavy dependencies (ratatui, etc.) have been removed for a leaner CLI experience.
> - **AI Explanation Caching:** Added a caching layer in `src/cache.rs` that stores AI-generated explanations for diffs, reducing API costs and providing better context for commit messages.
> - **Enhanced Configuration:** Added support for `.lumenignore` files, verbose logging (`--verbose`), and the ability to view configuration via `lumen configure --show`.
> - **Modernized:** Upgraded to Rust Edition 2024.
> 
> **Note:** These modifications, including the updates to this documentation, were implemented by AI using the `gemini-cli` agent.

A command-line tool that uses AI to streamline your git workflow - generate commit messages and explain changes.

## Table of Contents
- [Features](#features-)
- [Getting Started](#getting-started-)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Configuration (for AI features)](#configuration-for-ai-features)
- [Usage](#usage-)
  - [Generate Commit Messages](#generate-commit-messages)
  - [Generate Git Commands](#generate-git-commands)
  - [Explain Changes](#explain-changes)
  - [Interactive Mode](#interactive-mode)
  - [Tips & Tricks](#tips--tricks)
- [AI Providers](#ai-providers-)
- [Advanced Configuration](#advanced-configuration-)
  - [Configuration File](#configuration-file)
  - [Configuration Precedence](#configuration-precedence)
- [Attribution](#attribution)

## Features 🔅

- **Smart Commit Messages**: Generate conventional commit messages for your staged changes
- **Explanation Caching**: Automatically caches AI explanations for diffs, reducing API costs and providing better context
- **Git History Insights**: Understand what changed in any commit, branch, or your current work
- **Interactive Search**: Find and explore commits using fuzzy search
- **Change Analysis**: Ask questions about specific changes and their impact
- **Multiple AI Providers**: Supports OpenAI, Claude, Gemini, Groq, Ollama, and more
- **Highly Configurable**: Supports `.lumenignore` for custom file exclusions and verbose logging for debugging
- **Rich Output**: Markdown support for readable explanations (optional: mdcat)

## Getting Started 🔅

### Prerequisites
Before you begin, ensure you have:
1. `git` installed on your system
2. [fzf](https://github.com/junegunn/fzf) (optional) - Required for `lumen list` command
3. [mdcat](https://github.com/swsnr/mdcat) (optional) - Required for pretty output formatting

### Installation

Ensure you have the latest [Rust toolchain](https://rustup.rs) installed.

``` shell
- 1. Clone the repo
git clone https://github.com/noau/lumen.git
cd lumen
- 2. Build and install
cargo install --path .
```

### Configuration (for AI features)

If you want to use AI-powered features (`explain`, `draft`, `list`, `operate`), run the interactive setup:

```bash
lumen configure
```

You can also view your current configuration (with masked API keys) using:

```bash
lumen configure --show
```

This will guide you through:
1. Selecting an AI provider
2. Entering your API key (optional if using environment variable)
3. Specifying a custom model name (optional - press Enter to use the default)

The configuration is saved to `~/.config/lumen/lumen.config.json`.


## Usage 🔅

### Generate Commit Messages

Create meaningful commit messages for your staged changes. Lumen uses cached explanations for faster generation and better context:

```bash
# Basic usage - generates a commit message based on staged changes
lumen draft
# Output: "feat(button.tsx): Update button color to blue"

# Add context for more meaningful messages
lumen draft --context "match brand guidelines"
# Output: "feat(button.tsx): Update button color to align with brand identity guidelines"
```


### Generate Git Commands

Ask Lumen to generate Git commands based on a natural language query:

```bash
lumen operate "squash the last 3 commits into 1 with the message 'squashed commit'"
# Output: git reset --soft HEAD~3 && git commit -m "squashed commit" [y/N]
```

The command will display an explanation of what the generated command does, show any warnings for potentially dangerous operations, and prompt for confirmation before execution.

### Explain Changes

Understand what changed and why. Explanations for your working directory are cached to avoid redundant AI calls:

```bash
# Explain current changes in your working directory
lumen explain                         # All changes
lumen explain --staged                # Only staged changes

# Explain specific commits
lumen explain HEAD                    # Latest commit
lumen explain abc123f                 # Specific commit
lumen explain HEAD~3..HEAD            # Last 3 commits
lumen explain main..feature/A         # Branch comparison
lumen explain main...feature/A        # Branch comparison (merge base)

# Ask specific questions about changes
lumen explain --query "What's the performance impact of these changes?"
lumen explain HEAD --query "What are the potential side effects?"
```

### Interactive Mode
```bash
# Launch interactive fuzzy finder to search through commits (requires: fzf)
lumen list
```

### Tips & Tricks

```bash
# Copy commit message to clipboard
lumen draft | pbcopy                  # macOS
lumen draft | xclip -selection c      # Linux

# Disable markdown rendering if needed
lumen explain --no-mdcat

# Enable verbose logging for debugging
lumen --verbose explain
lumen --log-target <file-path> --verbose draft

# Open in your favorite editor
lumen draft | code -      

# Directly commit using the generated message
lumen draft | git commit -F -           
```

If you are using [lazygit](https://github.com/jesseduffield/lazygit), you can add this to the [user config](https://github.com/jesseduffield/lazygit/blob/master/docs/Config.md)
```yml
customCommands:
  - key: '<c-l>'
    context: 'files'
    command: 'lumen draft | tee >(pbcopy)'
    loadingText: 'Generating message...'
    showOutput: true
  - key: '<c-k>'
    context: 'files'
    command: 'lumen draft -c {{.Form.Context | quote}} | tee >(pbcopy)'
    loadingText: 'Generating message...'
    showOutput: true
    prompts:
          - type: 'input'
            title: 'Context'
            key: 'Context'
```

## AI Providers 🔅

Configure your preferred AI provider:

```bash
# Using CLI arguments
lumen -p openai -k "your-api-key" -m "gpt-5-mini" draft

# Using environment variables
export LUMEN_AI_PROVIDER="openai"
export LUMEN_API_KEY="your-api-key"
export LUMEN_AI_MODEL="gpt-5-mini"
```

### Supported Providers

| Provider | API Key Required | Models |
|----------|-----------------|---------|
| [OpenAI](https://platform.openai.com/docs/models) `openai` (Default) | Yes | `gpt-5.2`, `gpt-5`, `gpt-5-mini`, `gpt-5-nano`, `gpt-4.1`, `gpt-4.1-mini`, `o4-mini` (default: `gpt-5-mini`) |
| [Claude](https://www.anthropic.com/pricing) `claude` | Yes | `claude-sonnet-4-5-20250930`, `claude-opus-4-5-20251115`, `claude-haiku-4-5-20251015` (default: `claude-sonnet-4-5-20250930`) |
| [Gemini](https://ai.google.dev/) `gemini` | Yes (free tier) | `gemini-3-pro`, `gemini-3-flash-preview`, `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite` (default: `gemini-2.5-flash`) |
| [Groq](https://console.groq.com/docs/models) `groq` | Yes (free) | `llama-3.3-70b-versatile`, `llama-3.1-8b-instant`, `meta-llama/llama-4-maverick-17b-128e-instruct`, `openai/gpt-oss-120b` (default: `llama-3.3-70b-versatile`) |
| [DeepSeek](https://www.deepseek.com/) `deepseek` | Yes | `deepseek-chat` (V3.2), `deepseek-reasoner` (default: `deepseek-chat`) |
| [xAI](https://x.ai/) `xai` | Yes | `grok-4`, `grok-4-mini`, `grok-4-mini-fast` (default: `grok-4-mini-fast`) |
| [Ollama](https://github.com/ollama/ollama) `ollama` | No (local) | [see list](https://ollama.com/library) (default: `llama3.2`) |
| [OpenRouter](https://openrouter.ai/) `openrouter` | Yes | [see list](https://openrouter.ai/models) (default: `anthropic/claude-sonnet-4.5`) |
| [Vercel AI Gateway](https://vercel.com/docs/ai-gateway) `vercel` | Yes | [see list](https://vercel.com/docs/ai-gateway/supported-models) (default: `anthropic/claude-sonnet-4.5`) |

## Advanced Configuration 🔅

### .lumenignore
Lumen supports a `.lumenignore` file in your project root to exclude specific files or directories from being processed by the AI (e.g., lock files, large assets, or sensitive data). It follows standard gitignore pattern matching.

``` gitignore
.zed/
.idea/
GEMINI.md
Cargo.lock
```

### Configuration File
Lumen supports configuration through a JSON file. You can place the configuration file in one of the following locations:

1. Project Root: Create a lumen.config.json file in your project's root directory.
2. Custom Path: Specify a custom path using the --config CLI option.
3. Global Configuration (Optional): Place a lumen.config.json file in your system's default configuration directory:
    - Linux/macOS: `~/.config/lumen/lumen.config.json`
    - Windows: `%USERPROFILE%\.config\lumen\lumen.config.json`

Lumen will load configurations in the following order of priority:

1. CLI arguments (highest priority)
2. Configuration file specified by --config
3. Project root lumen.config.json
4. Global configuration file (lowest priority)

```json
{
  "provider": "openai",
  "model": "gpt-5-mini",
  "api_key": "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
  "draft": {
    "commit_types": {
      "docs": "Documentation only changes",
      "style": "Changes that do not affect the meaning of the code",
      "refactor": "A code change that neither fixes a bug nor adds a feature",
      "perf": "A code change that improves performance",
      "test": "Adding missing tests or correcting existing tests",
      "build": "Changes that affect the build system or external dependencies",
      "ci": "Changes to our CI configuration files and scripts",
      "chore": "Other changes that don't modify src or test files",
      "revert": "Reverts a previous commit",
      "feat": "A new feature",
      "fix": "A bug fix"
    }
  }
}
```

### Configuration Precedence
Options are applied in the following order (highest to lowest priority):
1. CLI Flags
2. Configuration File
3. Environment Variables
4. Default options

Example: Using different providers for different projects:
```bash
# Set global defaults in .zshrc/.bashrc
export LUMEN_AI_PROVIDER="openai"
export LUMEN_AI_MODEL="gpt-5-mini"
export LUMEN_API_KEY="sk-xxxxxxxxxxxxxxxxxxxxxxxx"

# Override per project using config file
{
  "provider": "ollama",
  "model": "llama3.2"
}

# Or override using CLI flags
lumen -p "ollama" -m "llama3.2" draft
```

## Attribution

This project is a fork of [jnsahaj/lumen](https://github.com/jnsahaj/lumen).

The original project and its contributors laid the foundation for this work.
For the full list of original contributors, please see:
[Original Contributors](https://github.com/jnsahaj/lumen/graphs/contributors).
