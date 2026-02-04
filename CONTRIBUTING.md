# Contributing to M.A.R.I.D

We welcome contributions to M.A.R.I.D (Market Analysis Research Intelligence Decisioning)!

## Code Style

*   Use `cargo fmt` to format your code.
*   Run `cargo clippy` to catch common issues.
*   Ensure all crates compile: `cargo build`.

## Architecture

*   **Strict Separation**:
    *   **Providers** (`crates/providers/*`) interact with the outside world (API calls).
    *   **Tools** (`crates/tools`) wrap providers and expose functionality to the system.
    *   **Agents** (`crates/agents`) orchestrate tools.
    *   **State** (`crates/storage`) manages the database.
*   **ToolRunner**: All tool execution MUST go through the `ToolRunner`. Do not bypass it to call providers directly from agents or the UI. This ensures caching and logging are preserved.

## Adding a New Tool

1.  Create a struct implementing the `Tool` trait in `crates/tools`.
2.  Define the `ToolSpec` (input/output schema).
3.  Implement `run`.
4.  Register the tool in `apps/daemon/src/main.rs`.

## Adding a New Agent

1.  Create a struct implementing the `Agent` trait in `crates/agents`.
2.  Implement `run`. Remember: Agents cannot do network I/O directly. Call `runner.run("tool_id", args)`.
3.  Register the agent in `apps/daemon/src/main.rs`.
