# Tools

Tools are discrete capabilities that the system (and the AI Brain) can execute.

## System

### `sys.status`
*   **Description**: Check system health and provider status.
*   **Returns**: System operational status and active providers.

### `sys.tools.list`
*   **Description**: List all available tools and their specifications.
*   **Returns**: JSON array of registered tools.

### `sys.agents.list`
*   **Description**: List all available high-level agents.
*   **Returns**: JSON array of registered agents.

## Web

### `web.search`
*   **Description**: Search the internet using Google Custom Search.
*   **Inputs**:
    *   `query` (string): The search term.
    *   `limit` (integer): Max results (default 5).
*   **Returns**: List of search results (Title, URL, Snippet).
*   **Requires**: `GOOGLE_CSE_API_KEY` and `GOOGLE_CSE_CX`.

### `web.fetch`
*   **Description**: Fetch a URL, extract readable text, and cache it.
*   **Inputs**:
    *   `url` (string): The URL to fetch.
*   **Returns**: The extracted main content of the page.
*   **Features**:
    *   Uses `readability` to strip boilerplate.
    *   Caches content in `web_pages` table (indefinite TTL unless manually cleared).
    *   Respects concurrency limits (default 2 concurrent fetches).
