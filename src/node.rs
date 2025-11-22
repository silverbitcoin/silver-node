//! # SilverBitcoin Node
//!
//! Main node implementation coordinating all subsystems.

use crate::config::NodeConfig;
use crate::genesis::GenesisConfig;
use crate::metrics::MetricsExporter;
use crate::health::{HealthCheckServer, HealthState, HealthMonitor, HealthStatus, SyncStatus};
use crate::resources::{ResourceMonitor, ResourceThresholds};
use silver_storage::ObjectStore;
use silver_network::NetworkNode;
use silver_consensus::MercuryConsensus;
use silver_execution::TransactionExecutor;
use silver_api::ApiServer;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, error, debug};

/// Node error types
#[derive(Error, Debug)]
pub enum NodeError {
    /// Storage initialization error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Network initialization error
    #[error("Network error: {0}")]
    #[allow(dead_code)]
    NetworkError(String),

    /// Consensus initialization error
    #[error("Consensus error: {0}")]
    #[allow(dead_code)]
    ConsensusError(String),

    /// Execution initialization error
    #[error("Execution error: {0}")]
    #[allow(dead_code)]
    ExecutionError(String),

    /// API initialization error
    #[error("API error: {0}")]
    ApiError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Genesis error
    #[error("Genesis error: {0}")]
    #[allow(dead_code)]
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

    /// Storage subsystem
    storage: Arc<ObjectStore>,

    /// Network subsystem
    network: Arc<NetworkNode>,

    /// Consensus subsystem
    consensus: Arc<MercuryConsensus>,

    /// Execution subsystem
    execution: Arc<TransactionExecutor>,

    /// API subsystem
    api: Arc<ApiServer>,

    /// Metrics exporter
    metrics: Arc<MetricsExporter>,

    /// Health check server
    health: Arc<HealthCheckServer>,

    /// Health state
    health_state: HealthState,

    /// Resource monitor
    resource_monitor: Arc<ResourceMonitor>,

    /// Shutdown signal
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
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
        
        // Initialize actual storage subsystem
        let db = Arc::new(silver_storage::RocksDatabase::open(&self.config.storage.db_path)
            .map_err(|e| NodeError::StorageError(format!("Failed to open database: {}", e)))?);
        
        self.storage = Arc::new(ObjectStore::new(db));
        
        info!("Storage subsystem initialized successfully");
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

        // Create genesis snapshot
        let genesis_snapshot = silver_core::Snapshot::genesis(
            genesis.chain_id.clone(),
            genesis.protocol_version.clone(),
        );
        
        // Initialize validator set
        let mut validator_set = silver_consensus::ValidatorSet::new();
        for validator in genesis.validators() {
            validator_set.add_validator(validator)?;
        }

        // Create initial accounts for validators
        for validator in genesis.validators() {
            let account = silver_core::Object::new_account(
                validator.address.clone(),
                genesis.initial_supply / genesis.validator_count() as u64,
            );
            self.storage.store_object(&account)?;
        }

        // Store genesis snapshot
        self.storage.store_snapshot(&genesis_snapshot)?;

        info!("Genesis state initialized successfully");
        Ok(())
    }

    /// Load existing blockchain state
    async fn load_existing_state(&self) -> Result<()> {
        info!("Loading existing blockchain state");

        // Load latest snapshot
        let latest_snapshot = self.storage.get_latest_snapshot()
            .map_err(|e| NodeError::StorageError(format!("Failed to load latest snapshot: {}", e)))?;

        if let Some(snapshot) = latest_snapshot {
            info!(
                "Loaded snapshot #{} from {}",
                snapshot.sequence_number,
                snapshot.timestamp
            );

            // Load validator set from snapshot
            let validator_set = self.storage.get_validator_set(snapshot.sequence_number)
                .map_err(|e| NodeError::StorageError(format!("Failed to load validator set: {}", e)))?;

            info!("Loaded {} validators", validator_set.len());

            // Initialize from last checkpoint
            debug!("Blockchain state loaded successfully");
        } else {
            return Err(NodeError::StorageError(
                "No snapshots found in storage".to_string(),
            ));
        }

        Ok(())
    }

    /// Initialize network subsystem
    async fn init_network(&mut self) -> Result<()> {
        info!("Network listening on: {}", self.config.network.listen_address);
        info!("P2P address: {}", self.config.network.p2p_address);
        info!("External address: {}", self.config.network.external_address);
        info!("Max peers: {}", self.config.network.max_peers);
        info!("Seed nodes: {}", self.config.network.seed_nodes.len());

        // Initialize actual network subsystem
        self.network = Arc::new(NetworkNode::new(
            self.config.network.clone(),
            self.storage.clone(),
        ).await
            .map_err(|e| NodeError::NetworkError(format!("Failed to initialize network: {}", e)))?);

        info!("Network subsystem initialized successfully");
        Ok(())
    }

    /// Initialize consensus subsystem
    async fn init_consensus(&mut self) -> Result<()> {
        info!("Snapshot interval: {}ms", self.config.consensus.snapshot_interval_ms);
        info!("Max batch transactions: {}", self.config.consensus.max_batch_transactions);
        info!("Max batch size: {} bytes", self.config.consensus.max_batch_size_bytes);

        let validator_id = if self.config.consensus.is_validator {
            if let Some(key_path) = &self.config.consensus.validator_key_path {
                info!("Loading validator key from: {:?}", key_path);
                // Load validator keys from file
                let key_data = std::fs::read_to_string(key_path)
                    .map_err(|e| NodeError::ConsensusError(format!("Failed to read validator key: {}", e)))?;
                
                let keypair = silver_crypto::KeyPair::from_json(&key_data)
                    .map_err(|e| NodeError::ConsensusError(format!("Failed to parse validator key: {}", e)))?;
                
                Some(keypair.address())
            } else {
                return Err(NodeError::ConsensusError(
                    "Validator mode enabled but no key path provided".to_string(),
                ));
            }
        } else {
            None
        };

        if let Some(stake) = self.config.consensus.stake_amount {
            info!("Validator stake: {} SBTC", stake);
        }

        // Initialize actual consensus subsystem
        self.consensus = Arc::new(MercuryConsensus::new(
            self.config.consensus.clone(),
            self.storage.clone(),
            self.network.clone(),
            validator_id,
        ).map_err(|e| NodeError::ConsensusError(format!("Failed to initialize consensus: {}", e)))?);

        info!("Consensus subsystem initialized successfully");
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

        // Initialize actual execution subsystem
        self.execution = Arc::new(TransactionExecutor::new(
            self.storage.clone(),
            self.config.execution.clone(),
        ).map_err(|e| NodeError::ExecutionError(format!("Failed to initialize execution: {}", e)))?);

        info!("Execution subsystem initialized successfully");
        Ok(())
    }

    /// Initialize API subsystem
    async fn init_api(&mut self) -> Result<()> {
        info!("JSON-RPC address: {}", self.config.api.json_rpc_address);
        info!("WebSocket address: {}", self.config.api.websocket_address);
        info!("CORS enabled: {}", self.config.api.enable_cors);
        info!("Rate limit: {} req/s", self.config.api.rate_limit_per_second);
        info!("Max batch size: {}", self.config.api.max_batch_size);

        // Initialize actual API subsystem
        self.api = Arc::new(ApiServer::new(
            self.config.api.clone(),
            self.storage.clone(),
            self.execution.clone(),
            self.consensus.clone(),
        ).map_err(|e| NodeError::ApiError(format!("Failed to initialize API: {}", e)))?);

        info!("API subsystem initialized successfully");
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
        
        // Get current heights from subsystems
        let current_height = self.consensus.get_current_snapshot_number();
        let network_height = self.network.get_network_height().await.unwrap_or(0);

        // Update sync status
        self.health_state.set_sync_status(SyncStatus {
            is_synced: current_height >= network_height,
            current_height,
            network_height,
            sync_progress: if network_height > 0 {
                (current_height as f64 / network_height as f64) * 100.0
            } else {
                100.0
            },
        }).await;

        info!("Node is now running");

        // Start all subsystems
        self.network.start().await
            .map_err(|e| NodeError::NetworkError(format!("Failed to start network: {}", e)))?;
        
        self.consensus.start().await
            .map_err(|e| NodeError::ConsensusError(format!("Failed to start consensus: {}", e)))?;
        
        self.execution.start().await
            .map_err(|e| NodeError::ExecutionError(format!("Failed to start execution: {}", e)))?;
        
        self.api.start().await
            .map_err(|e| NodeError::ApiError(format!("Failed to start API: {}", e)))?;

        info!("All subsystems started successfully");
        Ok(())
    }

    /// Check if node needs to sync
    async fn check_sync_status(&self) -> Result<bool> {
        // Check if local state is behind network
        let current_height = self.consensus.get_current_snapshot_number();
        let network_height = self.network.get_network_height().await.unwrap_or(current_height);
        
        Ok(current_height < network_height)
    }

    /// Sync with network
    async fn sync_with_network(&self) -> Result<()> {
        info!("Syncing with network");
        
        // Implement state synchronization
        let current_height = self.consensus.get_current_snapshot_number();
        let network_height = self.network.get_network_height().await?;

        if current_height >= network_height {
            info!("Already synced with network");
            return Ok(());
        }

        // Download latest snapshot from peers
        let latest_snapshot = self.network.download_latest_snapshot().await?;
        
        // Verify snapshot signatures
        self.consensus.verify_snapshot(&latest_snapshot).await?;
        
        // Apply transactions from snapshot forward
        self.consensus.apply_snapshot(&latest_snapshot).await?;

        info!("Sync completed: {} -> {}", current_height, network_height);
        Ok(())
    }

    /// Get current node state
    pub async fn state(&self) -> NodeState {
        *self.state.read().await
    }

    /// Check if node is running
    #[allow(dead_code)]
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
        // Gracefully shutdown API server
        if let Some(api) = &self.api {
            api.shutdown().await
                .map_err(|e| NodeError::ApiError(format!("Failed to shutdown API: {}", e)))?;
        }
        self.api = Arc::new(ApiServer::new_empty());
        Ok(())
    }

    /// Shutdown execution subsystem
    async fn shutdown_execution(&mut self) -> Result<()> {
        // Wait for pending transactions to complete
        if let Some(executor) = &self.execution {
            executor.wait_for_pending_transactions(
                tokio::time::Duration::from_secs(30)
            ).await
                .map_err(|e| NodeError::ExecutionError(format!("Failed to wait for transactions: {}", e)))?;
        }
        self.execution = Arc::new(TransactionExecutor::new_empty());
        Ok(())
    }

    /// Shutdown consensus subsystem
    async fn shutdown_consensus(&mut self) -> Result<()> {
        // Finalize pending snapshots
        if let Some(consensus) = &self.consensus {
            consensus.finalize_pending_snapshots().await
                .map_err(|e| NodeError::ConsensusError(format!("Failed to finalize snapshots: {}", e)))?;
        }
        self.consensus = Arc::new(MercuryConsensus::new_empty());
        Ok(())
    }

    /// Shutdown network subsystem
    async fn shutdown_network(&mut self) -> Result<()> {
        // Close all peer connections
        if let Some(network) = &self.network {
            network.close_all_connections().await
                .map_err(|e| NodeError::NetworkError(format!("Failed to close connections: {}", e)))?;
        }
        self.network = Arc::new(NetworkNode::new_empty());
        Ok(())
    }

    /// Shutdown storage subsystem
    async fn shutdown_storage(&mut self) -> Result<()> {
        // Flush pending writes and close database
        if let Some(storage) = &self.storage {
            storage.flush()
                .map_err(|e| NodeError::StorageError(format!("Failed to flush storage: {}", e)))?;
        }
        self.storage = Arc::new(ObjectStore::new_empty());
        Ok(())
    }

    /// Get node configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    /// Get genesis configuration
    #[allow(dead_code)]
    pub fn genesis(&self) -> Option<&GenesisConfig> {
        self.genesis.as_ref()
    }

    /// Get health state
    #[allow(dead_code)]
    pub fn health_state(&self) -> &HealthState {
        &self.health_state
    }

    /// Get metrics exporter
    #[allow(dead_code)]
    pub fn metrics(&self) -> Option<&MetricsExporter> {
        self.metrics.as_ref()
    }

    /// Get resource monitor
    #[allow(dead_code)]
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
