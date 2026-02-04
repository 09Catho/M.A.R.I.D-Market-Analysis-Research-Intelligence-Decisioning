# TermBrain

TermBrain is a local-first, terminal-based research and analytics suite inspired by professional financial terminals. It combines real-time market data, news aggregation, web research capabilities, and an AI "brain" into a unified workspace.

## Features

*   **Local-First Architecture**: Runs entirely on your machine. Data is stored in a local SQLite database (`~/.termbrain/termbrain.db`).
*   **Dual TUI Interface**:
    *   `termbrain-live`: Always-on market monitoring (Quote Tape, Candle Charts).
    *   `termbrain-workbench`: Deep analysis, history exploration, and AI console.
*   **AI Brain (TBP/1)**: An integrated AI assistant (default: Gemini) that adheres to the strict **TermBrain Protocol v1** (TBP/1). It can call tools, browse the web, and perform research without hallucinating capabilities.
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

2.  **Set API Keys**:
    Export your API keys in your shell. See [Configuration & API Keys](#configuration--api-keys) below.

3.  **Check System Health**:
    Run the doctor command to verify your environment and API keys.

    ```bash
    cargo run --bin termbrain -- doctor
    ```

4.  **Start the Daemon**:
    The daemon orchestrates data ingestion, tool execution, and the AI brain. It must be running for the TUIs to work.

    ```bash
    cargo run --bin termbrain -- daemon
    ```

5.  **Launch the Live Terminal**:
    Open a new terminal window and start the Live TUI for real-time monitoring.

    ```bash
    cargo run --bin termbrain -- live
    ```

6.  **Launch the Workbench**:
    Open another terminal window and start the Workbench for analysis.

    ```bash
    cargo run --bin termbrain -- workbench
    ```

## Configuration & API Keys

TermBrain uses environment variables for sensitive API keys. You should export these in your shell profile (e.g., `.bashrc`, `.zshrc`, or Windows System Properties).

### AI Providers (Gemini Default)

TermBrain defaults to Google Gemini.

*   **`GEMINI_API_KEY`**: (Recommended) API Key for Google Gemini.
    *   Get a key from [Google AI Studio](https://aistudio.google.com/app/apikey).
    *   **Model Selection**: You can change the model (e.g., `gemini-1.5-pro`, `gemini-2.0-flash`) in `~/.termbrain/config.json` under `brain.model`.

*   `OPENAI_API_KEY`: (Backup) If `GEMINI_API_KEY` is not set, TermBrain will try OpenAI.

### Web Research (Google Custom Search)
To enable `web.search` capabilities, you need a Google Programmable Search Engine (CSE).

1.  Create a Project in Google Cloud Console and enable the **Custom Search API**.
2.  Create an API Key and set it as `GOOGLE_CSE_API_KEY`.
3.  Create a Search Engine at [programmablesearchengine.google.com](https://programmablesearchengine.google.com/).
4.  Get the Search Engine ID (CX) and set it as `GOOGLE_CSE_CX`.

#### How to Set Keys

**Linux / macOS:**
Add these lines to your `~/.bashrc` or `~/.zshrc`:
```bash
export GEMINI_API_KEY="AIzaSy..."
export GOOGLE_CSE_API_KEY="AIzaSy..."
export GOOGLE_CSE_CX="0123456789..."
```
Then run `source ~/.bashrc`.

**Windows (PowerShell):**
```powershell
$env:GEMINI_API_KEY="AIzaSy..."
$env:GOOGLE_CSE_API_KEY="AIzaSy..."
$env:GOOGLE_CSE_CX="0123456789..."
```

## Tools & Agents

The system comes with built-in Tools (single capabilities) and Agents (workflows). You can run them manually via CLI to test.

### Available Tools

*   `sys.status`: Check system health.
*   `sys.tools.list`: List all tools.
*   `web.search`: Search Google (requires keys).
    *   Args: `{"query": "..."}`
*   `web.fetch`: Scrape a URL and extract text.
    *   Args: `{"url": "https://..."}`

**Example:**
```bash
termbrain tool run web.fetch '{"url": "https://rust-lang.org"}'
```

### Available Agents

*   `agent.brief`: Research a financial symbol or topic. It performs a web search and compiles a summary.
    *   Args: `{"symbol": "BTCUSDT"}` or `{"symbol": "Rust Language"}`

**Example:**
```bash
termbrain agent run agent.brief '{"symbol": "Ethereum"}'
```

## Architecture

TermBrain is built as a Rust workspace with the following components:

*   **Apps**: `daemon`, `live`, `workbench`, `cli`.
*   **Crates**: `tb-tools`, `tb-agents`, `tb-brain`, `tb-ipc`, `tb-storage`, `tb-providers-*`.

For more details, see [docs/architecture.md](docs/architecture.md).
