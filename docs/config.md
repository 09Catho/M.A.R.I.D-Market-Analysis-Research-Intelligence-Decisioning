# Configuration

TermBrain configuration is stored in `~/.termbrain/config.json`.

## Structure

```json
{
  "preset": "crypto",
  "default_symbol": "BTCUSDT",
  "timeframe": "1m",
  "providers": {
    "ai": {
      "chain": ["openai-main"],
      "timeout_seconds": 30
    },
    "search": {
      "provider": "google_cse"
    },
    "market": {
      "provider": "binance",
      "polling_interval_ms": 5000,
      "symbols": ["BTCUSDT", "ETHUSDT", "SOLUSDT"]
    },
    "news": {
      "feeds": [
        "https://cointelegraph.com/rss",
        "https://www.coindesk.com/arc/outboundfeeds/rss"
      ],
      "fetch_interval_seconds": 300
    }
  },
  "brain": {
    "model": "gpt-4o",
    "max_history_turns": 10
  }
}
```

## Environment Variables

Sensitive keys are NOT stored in `config.json`. They must be provided via environment variables.

| Variable | Description | Required For |
| :--- | :--- | :--- |
| `OPENAI_API_KEY` | OpenAI API Key (sk-...) | AI Brain |
| `GOOGLE_CSE_API_KEY` | Google Custom Search API Key | `web.search` tool |
| `GOOGLE_CSE_CX` | Google Custom Search Engine ID | `web.search` tool |
| `TERMRAIN_CONFIG` | Path to override config file | Optional |

## Mock Fallbacks

*   **Market**: If the configured market provider (Binance) fails (e.g., region blocking, API down), the system automatically degrades to a **Mock Provider** ("Random Walker"). This is indicated in the UI status bar as `MOCK`.
*   **AI**: If `OPENAI_API_KEY` is missing, the Brain degrades to a **Mock Responder** that returns static messages explaining the missing configuration.
