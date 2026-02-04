# TermBrain

TermBrain is a local-first, terminal-based research and analytics suite inspired by professional financial terminals. It combines real-time market data, news aggregation, web research capabilities, and an AI "brain" into a unified workspace.

## Features

*   **Local-First Architecture**: Runs entirely on your machine. Data is stored in a local SQLite database (`~/.termbrain/termbrain.db`).
*   **Dual TUI Interface**:
    *   `termbrain-live`: Always-on market monitoring (Quote Tape, Candle Charts).
    *   `termbrain-workbench`: Deep analysis, history exploration, and AI console.
*   **AI Brain (TBP/1)**: An integrated AI assistant that adheres to the strict **TermBrain Protocol v1** (TBP/1). It can call tools, browse the web, and perform research without hallucinating capabilities.
*   **Tool Registry**: A centralized system for executing capabilities (`web.search`, `web.fetch`, `market.quote`) with caching, concurrency control, and audit logging (receipts).
*   **Resilience**:
    *   **Market Fallback**: Automatically switches to a "Mock" provider if the primary API (Binance) is unreachable or restricted.
    *   **Graceful Degradation**: Tools like `web.search` return clear "Disabled" states if API keys are missing, preventing crashes.

## Installation

### Prerequisites

*   **Rust**: Ensure you have a recent stable version of Rust installed (via [rustup](https://rustup.rs/)).
*   **OpenSSL**: Required for HTTPS requests (`libssl-dev` on Ubuntu/Debian).

### Building

Clone the repository and build the workspace:

```bash
cargo build --release
```

The binaries will be located in `target/release/`.

## Quickstart

1.  **Initialize Configuration & Database**:
    Run the init command to create the default config at `~/.termbrain/config.json` and initialize the SQLite database.

    ```bash
    cargo run --bin termbrain -- init
    ```

2.  **Check System Health**:
    Run the doctor command to verify your environment and API keys.

    ```bash
    cargo run --bin termbrain -- doctor
    ```

3.  **Start the Daemon**:
    The daemon orchestrates data ingestion, tool execution, and the AI brain. It must be running for the TUIs to work.

    ```bash
    cargo run --bin termbrain -- daemon
    ```

4.  **Launch the Live Terminal**:
    Open a new terminal window and start the Live TUI for real-time monitoring.

    ```bash
    cargo run --bin termbrain -- live
    ```

5.  **Launch the Workbench**:
    Open another terminal window and start the Workbench for analysis.

    ```bash
    cargo run --bin termbrain -- workbench
    ```

## Configuration & API Keys

TermBrain uses environment variables for sensitive API keys. You should export these in your shell profile (e.g., `.bashrc` or `.zshrc`).

### AI Providers (Optional but Recommended)
*   `OPENAI_API_KEY`: Required for the AI Brain to function fully. If missing, it falls back to a Mock responder.
*   `ANTHROPIC_API_KEY`: (Planned) Backup provider.
*   `GEMINI_API_KEY`: (Planned) Backup provider.

### Web Research (Google Custom Search)
To enable `web.search` capabilities, you need a Google Programmable Search Engine (CSE).

1.  Create a Project in Google Cloud Console and enable the **Custom Search API**.
2.  Create an API Key and set it as `GOOGLE_CSE_API_KEY`.
3.  Create a Search Engine at [programmablesearchengine.google.com](https://programmablesearchengine.google.com/).
4.  Get the Search Engine ID (CX) and set it as `GOOGLE_CSE_CX`.

```bash
export OPENAI_API_KEY="sk-..."
export GOOGLE_CSE_API_KEY="AIza..."
export GOOGLE_CSE_CX="0123..."
```

## Architecture

TermBrain is built as a Rust workspace with the following components:

*   **Apps**:
    *   `daemon`: The core server (IPC, DB, Providers).
    *   `live`: Ratatui-based monitoring TUI.
    *   `workbench`: Ratatui-based analysis TUI.
    *   `cli`: Command-line interface for management.
*   **Crates**:
    *   `tb-tools`: The spine of the system. Handles tool execution, caching, and concurrency.
    *   `tb-agents`: Workflow engine for higher-level tasks.
    *   `tb-brain`: AI logic and TBP/1 protocol enforcement.
    *   `tb-ipc`: TCP-based Inter-Process Communication.
    *   `tb-storage`: SQLite schema and migrations.
    *   `tb-providers-*`: Modular implementations for Market, News, Web, and AI.

For more details, see [docs/architecture.md](docs/architecture.md).

## Usage

### CLI Tools

You can manually execute tools and agents via the CLI for testing or automation:

```bash
# Fetch a webpage
termbrain tool run web.fetch '{"url": "https://rust-lang.org"}'

# Run a market brief agent
termbrain agent run agent.brief '{"symbol": "BTCUSDT"}'
```

### Receipts & Audit Logs

Every action performed by the system is logged to the `tool_runs` table in the database. This includes:
*   **Input/Output**: The exact JSON arguments and results.
*   **Receipts**: Metadata about the execution (e.g., "Cache Hit", "Latency: 200ms", "Provider: Binance").
*   **Status**: Success, Error, or Timeout.

This ensures full auditability of the AI's actions.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on code style and architecture.
