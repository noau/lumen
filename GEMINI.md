# GEMINI.md

## Project Overview

This project, named "lumen," is a command-line tool written in Rust that enhances the user's git workflow with AI-powered features. It can generate commit messages, explain code changes, and provide a visual diff viewer in the terminal. The tool is designed to be flexible, supporting multiple AI providers like OpenAI, Claude, Gemini, and others.

The project is structured as a typical Rust application, with the main logic in the `src` directory. It uses the `clap` crate for command-line argument parsing, `reqwest` for making HTTP requests to AI APIs, and `ratatui` and `crossterm` for the terminal user interface. Syntax highlighting is implemented using `tree-sitter`.

Lumen also integrates with other command-line tools like `fzf` for interactive commit selection and `mdcat` for rendering Markdown in the terminal.

## Building and Running

To build the project, you need to have Rust and Cargo installed.

**Building:**
```bash
cargo build
```

**Running:**
You can run the application using `cargo run` with the desired command. For example:
```bash
# View the diff of the current changes
cargo run -- diff

# Generate a commit message for staged changes
cargo run -- draft
```

**Testing:**
The project does not have a readily apparent test suite in the file structure.

## Development Conventions

*   **Code Style:** The code follows standard Rust conventions.
*   **Commit Messages:** The tool itself is designed to generate conventional commit messages, so it's likely that the project follows this convention for its own commits.
*   **Error Handling:** The project uses a custom `LumenError` enum for error handling.
*   **Modularity:** The code is well-structured into modules, with each module responsible for a specific feature (e.g., `diff_ui`, `ai_prompt`, `git_entity`).
*   **Command Structure:** The application uses a `CommandType` enum to represent the different commands, and a `LumenCommand` struct to execute them. This provides a clear and organized way to manage the different functionalities of the tool.
*   **External Tool Integration:** The project is designed to integrate with other command-line tools like `fzf` and `mdcat` to enhance its functionality.
*   **AI Provider Abstraction:** The `LumenProvider` struct abstracts the interaction with different AI providers. It uses the `genai` crate to provide a unified interface to various AI models, including custom providers like OpenRouter and Vercel. This makes it easy to add support for new AI providers in the future.
*   **Prompt Engineering:** The `ai_prompt.rs` file is dedicated to building the prompts that are sent to the AI models. The prompts are carefully crafted to ensure that the AI-generated responses are accurate and concise. This is a key part of the application's AI-powered features.