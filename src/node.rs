//! # SilverBitcoin Node
//!
//! Main node implementation coordinating all subsystems.

use crate::config::NodeConfig;
use crate::genesis::GenesisConfig;
use crate::metrics::MetricsExporter;
use crate::health::{HealthCheckServer, HealthState, HealthMonitor, HealthStatus, SyncStatus};
use crate::resources::{ResourceMonitor, ResourceThresholds};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, error};

/// Node error types
#[derive(Error, Debug)]
pub enum NodeError {
    /// Storage initialization error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Network initialization error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Consensus initialization error
    #[error("Consensus error: {0}")]
    ConsensusError(String),

    /// Execution initialization error
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// API initialization error
    #[error("API error: {0}")]
    ApiError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Genesis error
    #[error("Genesis error: {0}")]
    GenesisError(String),

    /// Shutdown error
    #[error("Shutdown error: {0}")]
    ShutdownError(String),
}

/// Result type for node operations
pub type Result<T> = std::result::Result<T, NodeError>;

/// Node state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// Node is initializing
    Initializing,
    /// Node is syncing with network
    Syncing,
    /// Node is operational
    Running,
    /// Node is shutting down
    ShuttingDown,
    /// Node has stopped
    Stopped,
}

/// SilverBitcoin Node
pub struct SilverNode {
    /// Node configuration
    config: NodeConfig,

    /// Genesis configuration
    genesis: Option<GenesisConfig>,

    /// Node state
    state: Arc<RwLock<NodeState>>,

    /// Storage subsystem (placeholder for now)
    storage: Option<()>,

    /// Network subsystem (placeholder for now)
    network: Option<()>,

    /// Consensus subsystem (placeholder for now)
    consensus: Option<()>,

    /// Execution subsystem (placeholder for now)
    execution: Option<()>,

    /// API subsystem (placeholder for now)
    api: Option<()>,

    /// Metrics exporter
    metrics: Option<MetricsExporter>,

    /// Health check server
    health: Option<HealthCheckServer>,

    /// Health state
    health_state: HealthState,

    /// Resource monitor
    resource_monitor: Option<ResourceMonitor>,

    /// Shutdown signal
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

impl SilverNode {
    /// Create a new node instance
    pub fn new(config: NodeConfig, genesis: Option<GenesisConfig>) -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        let health_state = HealthState::new();
        
        Self {
            config,
            genesis,
            state: Arc::new(RwLock::new(NodeState::Initializing)),
            storage: None,
            network: None,
            consensus: None,
            execution: None,
            api: None,
            metrics: None,
            health: None,
            health_state,
            resource_monitor: None,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Initialize all subsystems
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing SilverBitcoin node");
        
        // Set state to initializing
        *self.state.write().await = NodeState::Initializing;

        // Initialize storage subsystem
        info!("Initializing storage subsystem");
        self.init_storage().await?;

        // Load or initialize genesis state
        if let Some(genesis) = &self.genesis {
            info!("Initializing genesis state for chain: {}", genesis.chain_id);
            self.init_genesis_state(genesis).await?;
        } else {
            info!("Loading existing blockchain state");
            self.load_existing_state().await?;
        }

        // Initialize network subsystem
        info!("Initializing network subsystem");
        self.init_network().await?;

        // Initialize consensus subsystem
        if self.config.consensus.is_validator {
            info!("Initializing consensus subsystem (validator mode)");
        } else {
            info!("Initializing consensus subsystem (full node mode)");
        }
        self.init_consensus().await?;

        // Initialize execution subsystem
        info!("Initializing execution subsystem with {} worker threads", 
              self.config.execution.worker_threads);
        self.init_execution().await?;

        // Initialize API subsystem
        info!("Initializing API subsystem");
        self.init_api().await?;

        // Initialize metrics subsystem
        if self.config.metrics.enable_metrics {
            info!("Initializing metrics subsystem");
            self.init_metrics().await?;
        }

        // Initialize health check subsystem
        info!("Initializing health check subsystem");
        self.init_health_check().await?;

        // Initialize resource monitoring
        info!("Initializing resource monitoring");
        self.init_resource_monitoring().await?;

        info!("Node initialization complete");
        Ok(())
    }

    /// Initialize storage subsystem
    async fn init_storage(&mut self) -> Result<()> {
        // Create data directory if it doesn't exist
        std::fs::create_dir_all(&self.config.storage.db_path)
            .map_err(|e| NodeError::StorageError(format!("Failed to create db directory: {}", e)))?;

        info!("Storage initialized at: {:?}", self.config.storage.db_path);
        info!("Object cache size: {} bytes", self.config.storage.object_cache_size);
        info!("Pruning enabled: {}", self.config.storage.enable_pruning);
        
        // TODO: Initialize actual storage subsystem
        // self.storage = Some(ObjectStore::new(&self.config.storage)?);
        
        self.storage = Some(());
        Ok(())
    }

    /// Initialize genesis state
    async fn init_genesis_state(&self, genesis: &GenesisConfig) -> Result<()> {
        info!("Initializing genesis state");
        info!("Chain ID: {}", genesis.chain_id);
        info!("Protocol version: {}.{}", genesis.protocol_version.major, genesis.protocol_version.minor);
        info!("Initial validators: {}", genesis.validator_count());
        info!("Total stake: {} SBTC", genesis.total_stake());
        info!("Initial supply: {} SBTC", genesis.initial_supply);

        // TODO: Initialize genesis state in storage
        // - Create genesis snapshot
        // - Initialize validator set
        // - Create initial accounts
        // - Set up initial objects

        Ok(())
    }

    /// Load existing blockchain state
    async fn load_existing_state(&self) -> Result<()> {
        info!("Loading existing blockchain state");

        // TODO: Load state from storage
        // - Load latest snapshot
        // - Load validator set
        // - Initialize from last checkpoint

        Ok(())
    }

    /// Initialize network subsystem
    async fn init_network(&mut self) -> Result<()> {
        info!("Network listening on: {}", self.config.network.listen_address);
        info!("P2P address: {}", self.config.network.p2p_address);
        info!("External address: {}", self.config.network.external_address);
        info!("Max peers: {}", self.config.network.max_peers);
        info!("Seed nodes: {}", self.config.network.seed_nodes.len());

        // TODO: Initialize actual network subsystem
        // self.network = Some(NetworkLayer::new(&self.config.network)?);

        self.network = Some(());
        Ok(())
    }

    /// Initialize consensus subsystem
    async fn init_consensus(&mut self) -> Result<()> {
        info!("Snapshot interval: {}ms", self.config.consensus.snapshot_interval_ms);
        info!("Max batch transactions: {}", self.config.consensus.max_batch_transactions);
        info!("Max batch size: {} bytes", self.config.consensus.max_batch_size_bytes);

        if self.config.consensus.is_validator {
            if let Some(key_path) = &self.config.consensus.validator_key_path {
                info!("Loading validator key from: {:?}", key_path);
                // TODO: Load validator keys
            }
            if let Some(stake) = self.config.consensus.stake_amount {
                info!("Validator stake: {} SBTC", stake);
            }
        }

        // TODO: Initialize actual consensus subsystem
        // self.consensus = Some(MercuryProtocol::new(&self.config.consensus)?);

        self.consensus = Some(());
        Ok(())
    }

    /// Initialize execution subsystem
    async fn init_execution(&mut self) -> Result<()> {
        info!("Worker threads: {}", self.config.execution.worker_threads);
        info!("NUMA-aware: {}", self.config.execution.numa_aware);
        info!("Fuel price: {} MIST/fuel", self.config.execution.fuel_price);

        // GPU configuration
        if self.config.gpu.enable_gpu {
            info!("GPU acceleration enabled");
            info!("GPU backend: {}", self.config.gpu.backend);
            info!("Min batch size for GPU: {}", self.config.gpu.min_batch_size);
        }

        // TODO: Initialize actual execution subsystem
        // self.execution = Some(ExecutionEngine::new(&self.config.execution)?);

        self.execution = Some(());
        Ok(())
    }

    /// Initialize API subsystem
    async fn init_api(&mut self) -> Result<()> {
        info!("JSON-RPC address: {}", self.config.api.json_rpc_address);
        info!("WebSocket address: {}", self.config.api.websocket_address);
        info!("CORS enabled: {}", self.config.api.enable_cors);
        info!("Rate limit: {} req/s", self.config.api.rate_limit_per_second);
        info!("Max batch size: {}", self.config.api.max_batch_size);

        // TODO: Initialize actual API subsystem
        // self.api = Some(ApiGateway::new(&self.config.api)?);

        self.api = Some(());
        Ok(())
    }

    /// Initialize metrics subsystem
    async fn init_metrics(&mut self) -> Result<()> {
        info!("Prometheus address: {}", self.config.metrics.prometheus_address);
        info!("Metrics update interval: {}s", self.config.metrics.update_interval_seconds);

        let shutdown_rx = self.shutdown_tx.as_ref()
            .ok_or(NodeError::ConfigError("Shutdown channel not initialized".to_string()))?
            .subscribe();

        let mut exporter = MetricsExporter::new(
            self.config.metrics.prometheus_address.clone(),
            self.config.metrics.update_interval_seconds,
            shutdown_rx,
        ).map_err(|e| NodeError::ApiError(format!("Failed to create metrics exporter: {}", e)))?;

        exporter.initialize().await
            .map_err(|e| NodeError::ApiError(format!("Failed to initialize metrics: {}", e)))?;

        exporter.start().await
            .map_err(|e| NodeError::ApiError(format!("Failed to start metrics server: {}", e)))?;

        exporter.start_update_loop().await;

        self.metrics = Some(exporter);
        info!("Metrics exporter started successfully");
        Ok(())
    }

    /// Initialize health check subsystem
    async fn init_health_check(&mut self) -> Result<()> {
        // Use metrics address but different port for health check
        let health_address = self.config.metrics.prometheus_address
            .replace(":9184", ":9185");
        
        info!("Health check address: {}", health_address);

        let shutdown_rx = self.shutdown_tx.as_ref()
            .ok_or(NodeError::ConfigError("Shutdown channel not initialized".to_string()))?
            .subscribe();

        let mut health_server = HealthCheckServer::new(
            health_address,
            self.health_state.clone(),
            shutdown_rx,
        );

        health_server.start().await
            .map_err(|e| NodeError::ApiError(format!("Failed to start health check server: {}", e)))?;

        // Start health monitor
        let health_monitor = HealthMonitor::new(
            self.health_state.clone(),
            self.config.metrics.update_interval_seconds,
        );
        health_monitor.start().await;

        self.health = Some(health_server);
        info!("Health check server started successfully");
        Ok(())
    }

    /// Initialize resource monitoring
    async fn init_resource_monitoring(&mut self) -> Result<()> {
        let thresholds = ResourceThresholds::default();
        
        let monitor = ResourceMonitor::new(
            thresholds,
            self.config.storage.db_path.clone(),
            self.config.metrics.update_interval_seconds,
        );

        monitor.start().await;

        self.resource_monitor = Some(monitor);
        info!("Resource monitoring started successfully");
        Ok(())
    }

    /// Start the node
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting SilverBitcoin node");

        // Check if node needs to sync
        let needs_sync = self.check_sync_status().await?;
        
        if needs_sync {
            info!("Node is behind, starting sync");
            *self.state.write().await = NodeState::Syncing;
            self.health_state.set_status(HealthStatus::Syncing).await;
            self.sync_with_network().await?;
        }

        // Set state to running
        *self.state.write().await = NodeState::Running;
        self.health_state.set_status(HealthStatus::Healthy).await;
        
        // Update sync status
        self.health_state.set_sync_status(SyncStatus {
            is_synced: true,
            current_height: 0, // TODO: Get from consensus
            network_height: 0, // TODO: Get from network
            sync_progress: 100.0,
        }).await;

        info!("Node is now running");

        // TODO: Start all subsystems
        // - Start network layer
        // - Start consensus engine
        // - Start execution engine
        // - Start API server

        Ok(())
    }

    /// Check if node needs to sync
    async fn check_sync_status(&self) -> Result<bool> {
        // TODO: Check if local state is behind network
        // For now, assume we're synced
        Ok(false)
    }

    /// Sync with network
    async fn sync_with_network(&self) -> Result<()> {
        info!("Syncing with network");
        
        // TODO: Implement state synchronization
        // - Download latest snapshot
        // - Verify snapshot signatures
        // - Apply transactions from snapshot forward

        Ok(())
    }

    /// Get current node state
    pub async fn state(&self) -> NodeState {
        *self.state.read().await
    }

    /// Check if node is running
    pub async fn is_running(&self) -> bool {
        *self.state.read().await == NodeState::Running
    }

    /// Shutdown the node gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down SilverBitcoin node");
        
        *self.state.write().await = NodeState::ShuttingDown;
        self.health_state.set_status(HealthStatus::Unhealthy).await;

        // Send shutdown signal to all subsystems
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        // Give subsystems time to receive shutdown signal
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Shutdown subsystems in reverse order
        info!("Stopping resource monitoring");
        self.resource_monitor = None;

        info!("Stopping health check");
        self.health = None;

        info!("Stopping metrics");
        self.metrics = None;

        info!("Stopping API subsystem");
        self.shutdown_api().await?;

        info!("Stopping execution subsystem");
        self.shutdown_execution().await?;

        info!("Stopping consensus subsystem");
        self.shutdown_consensus().await?;

        info!("Stopping network subsystem");
        self.shutdown_network().await?;

        info!("Persisting storage state");
        self.shutdown_storage().await?;

        *self.state.write().await = NodeState::Stopped;
        info!("Node shutdown complete");

        Ok(())
    }

    /// Shutdown API subsystem
    async fn shutdown_api(&mut self) -> Result<()> {
        // TODO: Gracefully shutdown API server
        self.api = None;
        Ok(())
    }

    /// Shutdown execution subsystem
    async fn shutdown_execution(&mut self) -> Result<()> {
        // TODO: Wait for pending transactions to complete
        self.execution = None;
        Ok(())
    }

    /// Shutdown consensus subsystem
    async fn shutdown_consensus(&mut self) -> Result<()> {
        // TODO: Finalize pending snapshots
        self.consensus = None;
        Ok(())
    }

    /// Shutdown network subsystem
    async fn shutdown_network(&mut self) -> Result<()> {
        // TODO: Close all peer connections
        self.network = None;
        Ok(())
    }

    /// Shutdown storage subsystem
    async fn shutdown_storage(&mut self) -> Result<()> {
        // TODO: Flush pending writes, close database
        self.storage = None;
        Ok(())
    }

    /// Get node configuration
    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    /// Get genesis configuration
    pub fn genesis(&self) -> Option<&GenesisConfig> {
        self.genesis.as_ref()
    }

    /// Get health state
    pub fn health_state(&self) -> &HealthState {
        &self.health_state
    }

    /// Get metrics exporter
    pub fn metrics(&self) -> Option<&MetricsExporter> {
        self.metrics.as_ref()
    }

    /// Get resource monitor
    pub fn resource_monitor(&self) -> Option<&ResourceMonitor> {
        self.resource_monitor.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NodeConfig;

    #[tokio::test]
    async fn test_node_creation() {
        let config = NodeConfig::default();
        let node = SilverNode::new(config, None);
        assert_eq!(node.state().await, NodeState::Initializing);
    }

    #[tokio::test]
    async fn test_node_state_transitions() {
        let config = NodeConfig::default();
        let node = SilverNode::new(config, None);
        
        assert_eq!(node.state().await, NodeState::Initializing);
        
        *node.state.write().await = NodeState::Running;
        assert_eq!(node.state().await, NodeState::Running);
        assert!(node.is_running().await);
    }
}
