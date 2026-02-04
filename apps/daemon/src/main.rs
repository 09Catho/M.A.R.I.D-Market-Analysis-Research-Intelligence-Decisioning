use tb_ipc::server::IpcServer;
use tb_ipc::DEFAULT_PORT;
use tracing_subscriber::EnvFilter;
use tb_config::{load_config, default_config_path};
use tb_storage::init_db;
use tb_tools::{ToolRunner, system::{SysToolsList, SysStatus}, web::{WebSearchTool, WebFetchTool}};
use tb_agents::{AgentRunner, SysAgentsList};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    tracing::info!("Starting TermBrain Daemon...");

    // Load Config
    let config_path = default_config_path();
    if !config_path.exists() {
        anyhow::bail!("Config not found at {:?}. Run 'termbrain init' first.", config_path);
    }
    let config = load_config(&config_path)?;

    // Init DB
    let db_path = config_path.parent().unwrap().join("termbrain.db");
    let pool = init_db(&db_path).await?;

    // Start IPC Server
    let server = IpcServer::new(DEFAULT_PORT);
    let ipc_tx = server.get_tx();

    // Providers
    tb_provider_market::start_market_service(config.providers.market.clone(), pool.clone(), ipc_tx.clone()).await;
    tb_provider_news::start_news_service(config.providers.news.clone(), pool.clone(), ipc_tx.clone()).await;

    // Tools
    let tool_runner = Arc::new(ToolRunner::new(pool.clone()));
    tool_runner.register(SysToolsList::new(tool_runner.clone())).await;
    tool_runner.register(SysStatus).await;

    // Web Tools
    let cse_key = std::env::var("GOOGLE_CSE_API_KEY").ok();
    let cse_cx = std::env::var("GOOGLE_CSE_CX").ok();
    tool_runner.register(WebSearchTool::new(cse_key, cse_cx)).await;
    tool_runner.register(WebFetchTool::new()).await;

    // Agents
    let agent_runner = Arc::new(AgentRunner::new(tool_runner.clone(), pool.clone()));
    tool_runner.register(SysAgentsList::new(agent_runner.clone())).await;

    // Run IPC
    server.run().await?;

    Ok(())
}
