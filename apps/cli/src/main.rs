use clap::{Parser, Subcommand};
use std::process::Command;
use tb_config::{save_config, default_config_path, Config};
use std::path::PathBuf;
use tb_tools::{ToolRunner, system::{SysToolsList, SysStatus}, web::{WebSearchTool, WebFetchTool}};
use tb_agents::{AgentRunner, SysAgentsList, brief::BriefAgent};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "termbrain")]
#[command(about = "TermBrain Terminal Suite", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Doctor,
    Daemon,
    Live,
    Workbench,
    TestIpc,
    #[command(subcommand)]
    Tool(ToolCommands),
    #[command(subcommand)]
    Agent(ToolCommands),
}

#[derive(Subcommand)]
enum ToolCommands {
    Run {
        id: String,
        args: String,
    },
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            let config_path = default_config_path();
            if config_path.exists() {
                println!("Config already exists at {:?}", config_path);
            } else {
                let config = Config::default();
                save_config(&config_path, &config)?;
                println!("Initialized config at {:?}", config_path);
            }

            // Init DB
            let db_path = get_db_path();
            println!("Initializing database at {:?}", db_path);
            match tb_storage::init_db(&db_path).await {
                Ok(_) => println!("✅ Database initialized."),
                Err(e) => eprintln!("❌ Failed to initialize database: {}", e),
            }
        }
        Commands::Doctor => {
             println!("Running doctor...");
             // Validate config
             let path = default_config_path();
             if !path.exists() {
                 eprintln!("❌ Config not found at {:?}. Run 'termbrain init' first.", path);
             } else {
                 match tb_config::load_config(&path) {
                     Ok(_) => println!("✅ Config valid."),
                     Err(e) => eprintln!("❌ Config invalid: {}", e),
                 }
             }

             // Validate DB
             let db_path = get_db_path();
             if !db_path.exists() {
                  eprintln!("❌ Database not found at {:?}. Run 'termbrain init'.", db_path);
             } else {
                  // Try connecting
                  match tb_storage::init_db(&db_path).await {
                      Ok(_) => println!("✅ Database connection successful."),
                      Err(e) => eprintln!("❌ Database connection failed: {}", e),
                  }
             }

             // Check env vars
             let vars = ["GEMINI_API_KEY", "OPENAI_API_KEY", "ANTHROPIC_API_KEY", "GOOGLE_CSE_API_KEY", "GOOGLE_CSE_CX"];
             for var in vars {
                 if std::env::var(var).is_ok() {
                     println!("✅ {} is set.", var);
                 } else {
                     if var == "GEMINI_API_KEY" {
                         println!("⚠️ {} is NOT set (Recommended Default).", var);
                     } else {
                         println!("⚠️ {} is NOT set.", var);
                     }
                 }
             }
        }
        Commands::Daemon => {
             run_binary("termbrain-daemon");
        }
        Commands::Live => {
             run_binary("termbrain-live");
        }
        Commands::Workbench => {
             run_binary("termbrain-workbench");
        }
        Commands::TestIpc => {
             println!("Testing IPC connection...");
             use tb_ipc::client::IpcClient;
             use tb_ipc::{Message, DEFAULT_PORT};

             let (client, mut rx) = IpcClient::connect(DEFAULT_PORT).await?;
             println!("Connected.");

             let req = Message::Request {
                 id: 1,
                 method: "ping".to_string(),
                 params: serde_json::json!({}),
             };
             client.send(req).await?;
             println!("Sent ping.");

             if let Some(msg) = rx.recv().await {
                 println!("Received: {:?}", msg);
             } else {
                 println!("Connection closed.");
             }
        }
        Commands::Tool(cmd) => {
             let db_path = get_db_path();
             let pool = tb_storage::init_db(&db_path).await?;
             let runner = Arc::new(ToolRunner::new(pool.clone()));

             runner.register(SysToolsList::new(runner.clone())).await;
             runner.register(SysStatus).await;

             let cse_key = std::env::var("GOOGLE_CSE_API_KEY").ok();
             let cse_cx = std::env::var("GOOGLE_CSE_CX").ok();
             runner.register(WebSearchTool::new(cse_key, cse_cx)).await;
             runner.register(WebFetchTool::new()).await;

             let agent_runner = Arc::new(AgentRunner::new(runner.clone(), pool.clone()));
             runner.register(SysAgentsList::new(agent_runner.clone())).await;

             match cmd {
                 ToolCommands::Run { id, args } => {
                     let json_args: serde_json::Value = serde_json::from_str(&args)?;
                     println!("Running {} with {}...", id, args);
                     match runner.run(&id, json_args).await {
                         Ok(output) => {
                             println!("Summary: {}", output.summary);
                             println!("Receipts: {}", serde_json::to_string_pretty(&output.receipts)?);
                         },
                         Err(e) => eprintln!("Error: {}", e),
                     }
                 },
                 ToolCommands::List => {
                     let tools = runner.list_tools().await;
                     for tool in tools {
                         println!("- {} ({:?}): {}", tool.id, tool.tags, tool.description);
                     }
                 }
             }
        }
        Commands::Agent(cmd) => {
             let db_path = get_db_path();
             let pool = tb_storage::init_db(&db_path).await?;
             let runner = Arc::new(ToolRunner::new(pool.clone()));

             let cse_key = std::env::var("GOOGLE_CSE_API_KEY").ok();
             let cse_cx = std::env::var("GOOGLE_CSE_CX").ok();
             runner.register(WebSearchTool::new(cse_key, cse_cx)).await;

             let agent_runner = Arc::new(AgentRunner::new(runner.clone(), pool.clone()));
             agent_runner.register(BriefAgent).await;

             match cmd {
                 ToolCommands::Run { id, args } => {
                     let json_args: serde_json::Value = serde_json::from_str(&args)?;
                     println!("Running agent {} with {}...", id, args);
                     match agent_runner.run(&id, json_args).await {
                         Ok(output) => {
                             println!("Summary: {}", output.summary);
                             println!("Receipts: {}", serde_json::to_string_pretty(&output.receipts)?);
                         },
                         Err(e) => eprintln!("Error: {}", e),
                     }
                 },
                 ToolCommands::List => {
                     let agents = agent_runner.list_agents().await;
                     for (id, desc) in agents {
                         println!("- {}: {}", id, desc);
                     }
                 }
             }
        }
    }
    Ok(())
}

fn get_db_path() -> PathBuf {
     let home = directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".termbrain").join("termbrain.db")
}

fn run_binary(bin_name: &str) {
    println!("Starting {}...", bin_name);

    // Try to find the binary in the same directory as the current executable
    let mut command = if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            let bin_path = parent.join(bin_name);
            if bin_path.exists() {
                Command::new(bin_path)
            } else {
                Command::new(bin_name)
            }
        } else {
            Command::new(bin_name)
        }
    } else {
        Command::new(bin_name)
    };

    let status = command.status();
    match status {
        Ok(s) => {
            if !s.success() {
                eprintln!("{} exited with status: {}", bin_name, s);
            }
        }
        Err(e) => {
             eprintln!("Failed to start {}: {}. Make sure it is installed or in target dir.", bin_name, e);
        }
    }
}
