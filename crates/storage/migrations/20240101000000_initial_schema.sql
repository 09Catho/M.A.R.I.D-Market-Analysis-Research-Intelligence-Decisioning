-- Core
CREATE TABLE IF NOT EXISTS app_meta (
    k TEXT PRIMARY KEY,
    v TEXT NOT NULL
);

-- We create this manually as requested, independent of sqlx's internal table
CREATE TABLE IF NOT EXISTS schema_migrations_custom (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL
);

-- Sessions
CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY,
    created_at TEXT,
    last_seen_at TEXT,
    client_kind TEXT CHECK(client_kind IN ('live','workbench'))
);

CREATE TABLE IF NOT EXISTS session_state (
    session_id TEXT PRIMARY KEY,
    active_symbol TEXT,
    active_tf TEXT,
    active_watchlist TEXT,
    active_portfolio_id INTEGER,
    cursor_ts TEXT,
    updated_at TEXT,
    FOREIGN KEY(session_id) REFERENCES sessions(session_id)
);

-- Symbols/Watchlists
CREATE TABLE IF NOT EXISTS symbols (
    symbol_id INTEGER PRIMARY KEY,
    provider TEXT,
    symbol TEXT,
    display_name TEXT,
    base_ccy TEXT,
    quote_ccy TEXT,
    price_scale INTEGER DEFAULT 8,
    UNIQUE(provider, symbol)
);

CREATE TABLE IF NOT EXISTS watchlists (
    watchlist_id INTEGER PRIMARY KEY,
    name TEXT UNIQUE,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS watchlist_items (
    watchlist_id INTEGER,
    symbol_id INTEGER,
    rank INTEGER DEFAULT 0,
    PRIMARY KEY(watchlist_id, symbol_id),
    FOREIGN KEY(watchlist_id) REFERENCES watchlists(watchlist_id),
    FOREIGN KEY(symbol_id) REFERENCES symbols(symbol_id)
);

-- Market
CREATE TABLE IF NOT EXISTS market_quotes (
    symbol_id INTEGER PRIMARY KEY,
    ts TEXT,
    last_i INTEGER,
    bid_i INTEGER,
    ask_i INTEGER,
    volume_i INTEGER,
    source TEXT,
    is_stale INTEGER DEFAULT 0,
    FOREIGN KEY(symbol_id) REFERENCES symbols(symbol_id)
);

CREATE TABLE IF NOT EXISTS market_bars (
    symbol_id INTEGER,
    tf TEXT,
    ts TEXT,
    open_i INTEGER,
    high_i INTEGER,
    low_i INTEGER,
    close_i INTEGER,
    volume_i INTEGER,
    source TEXT,
    PRIMARY KEY(symbol_id, tf, ts),
    FOREIGN KEY(symbol_id) REFERENCES symbols(symbol_id)
);

-- News/Web
CREATE TABLE IF NOT EXISTS news_items (
    news_id INTEGER PRIMARY KEY,
    provider TEXT,
    title TEXT,
    url TEXT,
    published_at TEXT,
    fetched_at TEXT,
    content_text TEXT,
    content_hash TEXT,
    lang TEXT,
    symbol_hint TEXT
);

CREATE TABLE IF NOT EXISTS web_pages (
    page_id INTEGER PRIMARY KEY,
    url TEXT UNIQUE,
    title TEXT,
    fetched_at TEXT,
    status_code INTEGER,
    content_text TEXT,
    content_hash TEXT,
    canonical_url TEXT,
    extract_meta_json TEXT
);

-- Artifacts/Notes/Reports
CREATE TABLE IF NOT EXISTS artifacts (
    artifact_id TEXT PRIMARY KEY,
    kind TEXT,
    mime TEXT,
    path TEXT,
    sha256 TEXT,
    bytes INTEGER,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS notes (
    note_id INTEGER PRIMARY KEY,
    scope TEXT CHECK(scope IN ('global','symbol','portfolio')),
    symbol_id INTEGER,
    portfolio_id INTEGER,
    title TEXT,
    body TEXT,
    created_at TEXT,
    updated_at TEXT,
    FOREIGN KEY(symbol_id) REFERENCES symbols(symbol_id)
);

CREATE TABLE IF NOT EXISTS reports (
    report_id INTEGER PRIMARY KEY,
    kind TEXT,
    title TEXT,
    created_at TEXT,
    params_json TEXT,
    artifact_id TEXT,
    FOREIGN KEY(artifact_id) REFERENCES artifacts(artifact_id)
);

-- Chat
CREATE TABLE IF NOT EXISTS chat_threads (
    thread_id INTEGER PRIMARY KEY,
    title TEXT,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS chat_messages (
    msg_id INTEGER PRIMARY KEY,
    thread_id INTEGER,
    role TEXT CHECK(role IN ('user','assistant','system')),
    content TEXT,
    created_at TEXT,
    FOREIGN KEY(thread_id) REFERENCES chat_threads(thread_id)
);

-- Tools/Agents runs
CREATE TABLE IF NOT EXISTS tool_runs (
    tool_run_id TEXT PRIMARY KEY,
    tool_id TEXT,
    started_at TEXT,
    ended_at TEXT,
    status TEXT CHECK(status IN ('ok','error','timeout','running')),
    input_json TEXT,
    output_summary_json TEXT,
    receipts_json TEXT,
    error_text TEXT
);

CREATE TABLE IF NOT EXISTS tool_run_artifacts (
    tool_run_id TEXT,
    artifact_id TEXT,
    PRIMARY KEY(tool_run_id, artifact_id),
    FOREIGN KEY(tool_run_id) REFERENCES tool_runs(tool_run_id),
    FOREIGN KEY(artifact_id) REFERENCES artifacts(artifact_id)
);

CREATE TABLE IF NOT EXISTS agent_runs (
    agent_run_id TEXT PRIMARY KEY,
    agent_id TEXT,
    started_at TEXT,
    ended_at TEXT,
    status TEXT,
    input_json TEXT,
    output_summary_json TEXT,
    receipts_json TEXT
);

CREATE TABLE IF NOT EXISTS agent_run_tool_runs (
    agent_run_id TEXT,
    tool_run_id TEXT,
    PRIMARY KEY(agent_run_id, tool_run_id),
    FOREIGN KEY(agent_run_id) REFERENCES agent_runs(agent_run_id),
    FOREIGN KEY(tool_run_id) REFERENCES tool_runs(tool_run_id)
);

-- Portfolio
CREATE TABLE IF NOT EXISTS portfolios (
    portfolio_id INTEGER PRIMARY KEY,
    name TEXT UNIQUE,
    base_ccy TEXT,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS portfolio_transactions (
    tx_id INTEGER PRIMARY KEY,
    portfolio_id INTEGER,
    ts TEXT,
    type TEXT CHECK(type IN ('BUY','SELL','DEPOSIT','WITHDRAW','FEE','TRANSFER')),
    symbol_id INTEGER,
    qty_i INTEGER,
    price_i INTEGER,
    fee_i INTEGER,
    memo TEXT,
    FOREIGN KEY(portfolio_id) REFERENCES portfolios(portfolio_id),
    FOREIGN KEY(symbol_id) REFERENCES symbols(symbol_id)
);

CREATE TABLE IF NOT EXISTS portfolio_snapshots (
    snapshot_id INTEGER PRIMARY KEY,
    portfolio_id INTEGER,
    ts TEXT,
    cash_i INTEGER,
    total_value_i INTEGER,
    pnl_day_i INTEGER,
    artifact_id TEXT,
    FOREIGN KEY(portfolio_id) REFERENCES portfolios(portfolio_id),
    FOREIGN KEY(artifact_id) REFERENCES artifacts(artifact_id)
);

CREATE TABLE IF NOT EXISTS portfolio_positions (
    snapshot_id INTEGER,
    symbol_id INTEGER,
    qty_i INTEGER,
    avg_cost_i INTEGER,
    PRIMARY KEY(snapshot_id, symbol_id),
    FOREIGN KEY(snapshot_id) REFERENCES portfolio_snapshots(snapshot_id),
    FOREIGN KEY(symbol_id) REFERENCES symbols(symbol_id)
);

-- Risk
CREATE TABLE IF NOT EXISTS risk_snapshots (
    risk_id INTEGER PRIMARY KEY,
    portfolio_id INTEGER,
    ts TEXT,
    metrics_json TEXT,
    artifact_id TEXT,
    FOREIGN KEY(portfolio_id) REFERENCES portfolios(portfolio_id),
    FOREIGN KEY(artifact_id) REFERENCES artifacts(artifact_id)
);

CREATE TABLE IF NOT EXISTS scenario_runs (
    scenario_id INTEGER PRIMARY KEY,
    portfolio_id INTEGER,
    ts TEXT,
    scenario_json TEXT,
    result_json TEXT,
    artifact_id TEXT,
    FOREIGN KEY(portfolio_id) REFERENCES portfolios(portfolio_id),
    FOREIGN KEY(artifact_id) REFERENCES artifacts(artifact_id)
);

-- Alerts
CREATE TABLE IF NOT EXISTS alerts (
    alert_id INTEGER PRIMARY KEY,
    kind TEXT CHECK(kind IN ('price','news','anomaly')),
    enabled INTEGER DEFAULT 1,
    config_json TEXT,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS alert_events (
    event_id INTEGER PRIMARY KEY,
    alert_id INTEGER,
    fired_at TEXT,
    payload_json TEXT,
    acknowledged_at TEXT,
    FOREIGN KEY(alert_id) REFERENCES alerts(alert_id)
);

-- Memory + FTS
CREATE TABLE IF NOT EXISTS memory_items (
    mem_id INTEGER PRIMARY KEY,
    kind TEXT CHECK(kind IN ('note','report','news','web','chat')),
    ref_id TEXT,
    title TEXT,
    body TEXT,
    tags_json TEXT,
    created_at TEXT,
    updated_at TEXT
);

CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(title, body, content='memory_items', content_rowid='mem_id');

CREATE TRIGGER IF NOT EXISTS memory_items_ai AFTER INSERT ON memory_items BEGIN
  INSERT INTO memory_fts(rowid, title, body) VALUES (new.mem_id, new.title, new.body);
END;

CREATE TRIGGER IF NOT EXISTS memory_items_ad AFTER DELETE ON memory_items BEGIN
  INSERT INTO memory_fts(memory_fts, rowid, title, body) VALUES('delete', old.mem_id, old.title, old.body);
END;

CREATE TRIGGER IF NOT EXISTS memory_items_au AFTER UPDATE ON memory_items BEGIN
  INSERT INTO memory_fts(memory_fts, rowid, title, body) VALUES('delete', old.mem_id, old.title, old.body);
  INSERT INTO memory_fts(rowid, title, body) VALUES (new.mem_id, new.title, new.body);
END;

CREATE TABLE IF NOT EXISTS pins (
    pin_id INTEGER PRIMARY KEY,
    mem_id INTEGER,
    created_at TEXT,
    FOREIGN KEY(mem_id) REFERENCES memory_items(mem_id)
);
