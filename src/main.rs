//! # SilverBitcoin Node
//!
//! Main blockchain node binary that coordinates all subsystems.

#![allow(missing_docs)] // Internal implementation details

mod config;
mod genesis;
mod node;
mod logging;
mod lifecycle;
mod metrics;
mod health;
mod resources;

use clap::Parser;
use std::path::PathBuf;
use tracing::{info, error};

use config::NodeConfig;
use genesis::GenesisConfig;
use node::SilverNode;
use lifecycle::LifecycleManager;

#[derive(Parser)]
#[command(name = "silver-node")]
#[command(about = "SilverBitcoin blockchain node - Fast, Secure, Accessible Bitcoin for Everyone", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to configuration file
    #[arg(short, long, default_value = "node.toml")]
    config: PathBuf,

    /// Genesis file path (required for new networks)
    #[arg(short, long)]
    genesis: Option<PathBuf>,

    /// Enable validator mode
    #[arg(short, long)]
    validator: bool,

    /// Data directory
    #[arg(short, long)]
    data_dir: Option<PathBuf>,

    /// Override log level (trace, debug, info, warn, error)
    #[arg(long)]
    log_level: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    dotenv::dotenv().ok();

    let cli = Cli::parse();

    // Load configuration
    let mut config = if cli.config.exists() {
        info!("Loading configuration from: {:?}", cli.config);
        NodeConfig::from_file(&cli.config)?
    } else {
        info!("Configuration file not found, using defaults");
        NodeConfig::default()
    };

    // Apply CLI overrides
    if let Some(data_dir) = cli.data_dir {
        config.storage.db_path = data_dir.join("db");
        config.logging.log_path = data_dir.join("logs/node.log");
    }

    if cli.validator {
        config.consensus.is_validator = true;
    }

    if let Some(log_level) = cli.log_level {
        config.logging.level = log_level;
    }

    // Initialize logging system
    logging::init_logging(&config.logging)?;

    info!("╔═══════════════════════════════════════════════════════════╗");
    info!("║         SilverBitcoin Blockchain Node v{}            ║", env!("CARGO_PKG_VERSION"));
    info!("║   Fast, Secure, Accessible Bitcoin for Everyone          ║");
    info!("╚═══════════════════════════════════════════════════════════╝");
    info!("");

    // Load genesis configuration if provided
    let genesis = if let Some(genesis_path) = cli.genesis {
        info!("Loading genesis configuration from: {:?}", genesis_path);
        Some(GenesisConfig::from_file(&genesis_path)?)
    } else {
        None
    };

    // Display configuration summary
    info!("Configuration Summary:");
    info!("  Network: {}", config.network.listen_address);
    info!("  Storage: {:?}", config.storage.db_path);
    info!("  Validator: {}", config.consensus.is_validator);
    info!("  API: {} (JSON-RPC), {} (WebSocket)", 
          config.api.json_rpc_address, 
          config.api.websocket_address);
    info!("  Metrics: {}", config.metrics.prometheus_address);
    info!("");

    // Create node instance
    let node = SilverNode::new(config, genesis);

    // Create lifecycle manager
    let mut lifecycle = LifecycleManager::new(node);

    // Run node with lifecycle management
    match lifecycle.run().await {
        Ok(()) => {
            info!("Node exited successfully");
            Ok(())
        }
        Err(e) => {
            error!("Node error: {}", e);
            Err(e.into())
        }
    }
}
