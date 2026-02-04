# Agents

Agents are higher-level workflows that orchestrate multiple tools to achieve a complex goal. They do NOT perform network I/O directly; they must call tools via the `ToolRunner`.

## `agent.brief`

*   **Description**: Generates a brief summary for a given symbol or topic.
*   **Workflow**:
    1.  Uses `web.search` to find recent news and information about the symbol.
    2.  Compiles the search snippets into a summary.
    3.  (Future) Will integrate `market.quote` for price data.
*   **Inputs**:
    *   `symbol` (string): The ticker or topic to research (e.g., "BTCUSDT", "Rust Lang").
*   **Output**: A text summary incorporating search findings.
