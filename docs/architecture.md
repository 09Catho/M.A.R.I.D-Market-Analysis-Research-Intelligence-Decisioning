# M.A.R.I.D Architecture

M.A.R.I.D (TermBrain) is designed as a modular, local-first system.

## High-Level Overview

```
+----------------+      +----------------+      +----------------+
|  TermBrain     |      |  TermBrain     |      |  TermBrain     |
|  Live (TUI)    |      |  Workbench     |      |  CLI           |
+-------+--------+      +-------+--------+      +-------+--------+
        | TCP (IPC)             | TCP (IPC)             | Direct/IPC
        v                       v                       v
+----------------------------------------------------------------+
|                       M.A.R.I.D Daemon                         |
|                                                                |
|  +-----------+   +-------------+   +-----------+   +--------+  |
|  | Providers |   | ToolRunner  |   | AI Brain  |   | Agents |  |
|  +-----------+   +------+------+   +-----------+   +--------+  |
|      |  ^               |               ^               ^      |
|      v  |               v               |               |      |
|  +----------------------------------------------------------+  |
|  |                     State (SQLite)                       |  |
|  +----------------------------------------------------------+  |
+----------------------------------------------------------------+
```

## Components

### 1. Daemon
The daemon is the heart of the system. It runs as a background process and handles:
*   **Providers**: Ingests data from external sources (Binance, RSS, Google).
*   **IPC Server**: Exposes a TCP socket (default 9000) for clients to subscribe to streams (quotes, news) or request actions (tool execution).
*   **ToolRunner**: The execution engine for all capabilities.

### 2. ToolRunner
The `ToolRunner` is the critical spine that ensures safety and reliability. It is responsible for:
*   **Execution**: Running the actual logic of a tool.
*   **Concurrency Control**: Using Semaphores to limit concurrent operations per domain (e.g., max 2 concurrent web fetches).
*   **Caching**: Generic DB-backed caching based on the hash of `(tool_id + input_args)`.
*   **Logging**: Persisting every run to the `tool_runs` table with full input/output and performance receipts.

### 3. AI Brain (TBP/1)
The AI Brain implements the **TermBrain Protocol v1 (TBP/1)**. This is a strict interaction loop:
1.  **Context Construction**: The brain assembles a prompt containing the user query, system status, and a dynamic catalog of available tools.
2.  **LLM Call**: It sends this to the provider (OpenAI).
3.  **Strict Parsing**: The response MUST be valid JSON conforming to the TBP/1 schema (`tool_call` or `final_response`). Arbitrary text is rejected or wrapped.
4.  **Tool Execution**: If a tool is called, the Daemon executes it via `ToolRunner` and feeds the result back to the Brain.

### 4. Storage
SQLite is used for all state.
*   `market_quotes` / `market_bars`: Financial time-series data.
*   `news_items`: Ingested headlines.
*   `web_pages`: Cached web content.
*   `tool_runs` / `agent_runs`: Audit logs.
*   `app_meta`: System configuration and migration state.

## IPC Protocol
The IPC layer uses line-delimited JSON over TCP.
*   **Requests**: `{ "kind": "Request", "id": 1, "method": "...", "params": {...} }`
*   **Responses**: `{ "kind": "Response", "id": 1, "result": ... }`
*   **Events**: `{ "kind": "Event", "topic": "market.quote", "payload": ... }`
