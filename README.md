# M.A.R.I.D
> **M**arket **A**nalysis **R**esearch **I**ntelligence **D**ecisioning

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/Status-Beta-orange.svg)]()

**M.A.R.I.D** is a professional-grade, local-first terminal workspace designed for deep financial research, real-time market monitoring, and AI-assisted analysis. It bridges the gap between a Bloomberg Terminal's data density and a modern AI researcher's workflow.

---

## 🚀 Vision

While the current release uses **Cryptocurrency** as its default data preset (via Binance), **M.A.R.I.D** is designed to be asset-agnostic. The core architecture—time-series storage, charting, and AI analysis—is built to handle:
*   **Stocks / Equities** (Planned)
*   **Forex / Commodities**
*   **Custom Alternative Data**

We are building the ultimate local research OS for *any* market.

---

## ✨ Key Features

### 🖥️ Dual-Terminal Interface
*   **Live Terminal**: A dedicated, distraction-free TUI for real-time monitoring. Features a high-speed Quote Tape and reactive Candle Charts.
*   **Workbench**: A deep-dive analytics cockpit. Explore historical data, run backtests, and converse with the AI brain.

### 🧠 AI Brain (TBP/1 Protocol)
An integrated AI that follows the strict **TermBrain Protocol v1**.
*   **No Hallucinations**: The AI cannot "pretend" to do things. It must strictly call registered Tools.
*   **Tool-Augmented**: Can browse the web (`web.search`), fetch reports, and analyze data on demand.
*   **Default Provider**: Optimized for **Google Gemini** (1.5 Pro), with failover to OpenAI.

### 🛡️ Robust & Local-First
*   **Privacy**: All data (keys, history, notes) lives in a local SQLite database (`~/.termbrain/termbrain.db`).
*   **Resilience**: Smart fallbacks ensure you never lose visibility. If an API goes down, M.A.R.I.D switches to Mock data automatically, clearly flagged in the UI.
*   **Tool Registry**: A centralized "spine" that handles caching, rate-limiting, and audit logging for every single action.

---

## 🔮 Roadmap: Phase 2 (Universal Markets)

The next major milestone involves expanding beyond the crypto default:

*   **Stock Market Integration**: First-class support for providers like **AlphaVantage**, **Polygon.io**, and **Yahoo Finance**.
*   **Forex Support**: Dedicated tools for currency pair analysis.
*   **Unified Symbol Search**: A global search bar ("AAPL", "BTC", "EURUSD") that auto-routes to the correct provider.
*   **Portfolio Import**: Import CSV ledgers from major brokerages.

---

## 🤖 Zero-Code Setup (AI IDEs)

Don't want to touch the command line? If you use **Cursor**, **Windsurf**, or **Claude Code**, copy and paste this prompt to have the AI set everything up for you:

> "I want to install and run 'M.A.R.I.D' (internal name: termbrain) on this machine. Act as a DevOps engineer and automate this:
> 1. Clone the repo (or use current dir).
> 2. Ensure `rustup` and `cargo` are installed.
> 3. Run `cargo build --release`.
> 4. Ask me for my **Gemini API Key** and **Google Custom Search Keys** (API+CX).
> 5. Securely export them in the shell.
> 6. Initialize the DB via `cargo run --bin termbrain -- init`.
> 7. Verify health via `cargo run --bin termbrain -- doctor`.
> 8. Launch the Daemon in the background and open the Live TUI."

---

## 📦 Installation

### Prerequisites
*   **Rust**: [Install via rustup](https://rustup.rs/)
*   **OpenSSL**: `sudo apt install libssl-dev` (Linux) or via Homebrew (macOS).

### Build & Run

```bash
# 1. Build the workspace
cargo build --release

# 2. Initialize Config & DB
./target/release/termbrain init

# 3. Check Status
./target/release/termbrain doctor

# 4. Start the Brain (Daemon)
./target/release/termbrain daemon

# 5. Launch UI (in new tabs)
./target/release/termbrain live
./target/release/termbrain workbench
```

---

## 🔑 Configuration & API Keys

M.A.R.I.D relies on environment variables for security.

### 1. AI Intelligence (Recommended)
We recommend **Google Gemini** for the best balance of speed and reasoning.
*   Get Key: [Google AI Studio](https://aistudio.google.com/app/apikey)
*   **Set Env**: `export GEMINI_API_KEY="AIza..."`

### 2. Web Research (Optional)
Enables the AI to browse the internet for news and reports.
*   **Keys**: `GOOGLE_CSE_API_KEY` and `GOOGLE_CSE_CX`.
*   [Setup Guide: Custom Search JSON API](https://developers.google.com/custom-search/v1/overview)

### 3. Market Data
*   **Default**: Uses public Binance API (no key required for basic data).
*   **Fallback**: Auto-mocks data if API is restricted in your region.

---

## 🛠️ Usage Examples

**Manual Tool Execution:**
```bash
# Fetch and read a webpage
termbrain tool run web.fetch '{"url": "https://finance.yahoo.com"}'
```

**Running an Agent Workflow:**
```bash
# Generate a market brief
termbrain agent run agent.brief '{"symbol": "NVDA"}'
```

---

## 🏗️ Architecture

M.A.R.I.D follows a modular workspace architecture:

*   `apps/daemon`: The central server and brain.
*   `apps/live`: Real-time monitoring TUI.
*   `apps/workbench`: Research TUI.
*   `crates/tb-tools`: The execution spine (caching/logging).
*   `crates/tb-brain`: The TBP/1 AI implementation.

See [docs/architecture.md](docs/architecture.md) for details.
