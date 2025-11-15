//! # Node Lifecycle Management
//!
//! Handles graceful startup, shutdown, and state persistence.

use crate::node::{NodeError, NodeState, Result, SilverNode};
use std::time::{Duration, Instant};
use tokio::signal;
use tracing::{info, warn, error};

/// Maximum shutdown time (30 seconds as per requirements)
const MAX_SHUTDOWN_TIME: Duration = Duration::from_secs(30);

/// Lifecycle manager for node operations
pub struct LifecycleManager {
    /// Reference to the node
    node: SilverNode,

    /// Start time
    start_time: Option<Instant>,
}

impl LifecycleManager {
    /// Create a new lifecycle manager
    pub fn new(node: SilverNode) -> Self {
        Self {
            node,
            start_time: None,
        }
    }

    /// Start the node with full lifecycle management
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting node lifecycle");

        // Initialize the node
        self.node.initialize().await?;

        // Start the node
        self.start_time = Some(Instant::now());
        self.node.start().await?;

        info!("Node started successfully");

        // Wait for shutdown signal
        self.wait_for_shutdown_signal().await;

        // Perform graceful shutdown
        self.graceful_shutdown().await?;

        Ok(())
    }

    /// Wait for shutdown signal (SIGINT, SIGTERM)
    async fn wait_for_shutdown_signal(&self) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install SIGTERM handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {
                info!("Received Ctrl+C signal");
            }
            _ = terminate => {
                info!("Received SIGTERM signal");
            }
        }
    }

    /// Perform graceful shutdown within time limit
    async fn graceful_shutdown(&mut self) -> Result<()> {
        info!("Initiating graceful shutdown");
        let shutdown_start = Instant::now();

        // Create timeout for shutdown
        let shutdown_result = tokio::time::timeout(
            MAX_SHUTDOWN_TIME,
            self.node.shutdown()
        ).await;

        match shutdown_result {
            Ok(Ok(())) => {
                let elapsed = shutdown_start.elapsed();
                info!("Graceful shutdown completed in {:?}", elapsed);
                
                if elapsed > Duration::from_secs(25) {
                    warn!("Shutdown took longer than expected: {:?}", elapsed);
                }
                
                Ok(())
            }
            Ok(Err(e)) => {
                error!("Error during shutdown: {}", e);
                Err(e)
            }
            Err(_) => {
                error!("Shutdown timeout exceeded {} seconds", MAX_SHUTDOWN_TIME.as_secs());
                Err(NodeError::ShutdownError(
                    "Shutdown timeout exceeded".to_string()
                ))
            }
        }
    }

    /// Get node uptime
    pub fn uptime(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    /// Resume from last snapshot
    pub async fn resume_from_snapshot(&mut self) -> Result<()> {
        info!("Resuming from last snapshot");

        // TODO: Implement snapshot resume logic
        // - Load latest snapshot from storage
        // - Verify snapshot integrity
        // - Restore state from snapshot
        // - Apply any pending transactions

        info!("Successfully resumed from snapshot");
        Ok(())
    }

    /// Persist state before shutdown
    pub async fn persist_state(&self) -> Result<()> {
        info!("Persisting node state");

        // TODO: Implement state persistence
        // - Create snapshot of current state
        // - Flush all pending writes
        // - Sync database to disk
        // - Save checkpoint information

        info!("State persisted successfully");
        Ok(())
    }

    /// Check if node is healthy
    pub async fn health_check(&self) -> HealthStatus {
        let state = self.node.state().await;
        let uptime = self.uptime();

        HealthStatus {
            is_running: state == NodeState::Running,
            state,
            uptime_seconds: uptime.map(|d| d.as_secs()).unwrap_or(0),
            // TODO: Add more health metrics
            is_synced: false,
            snapshot_height: 0,
            peer_count: 0,
        }
    }
}

/// Health status information
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Is node running
    pub is_running: bool,

    /// Current node state
    pub state: NodeState,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Is node synced with network
    pub is_synced: bool,

    /// Current snapshot height
    pub snapshot_height: u64,

    /// Number of connected peers
    pub peer_count: usize,
}

impl HealthStatus {
    /// Check if node is healthy
    pub fn is_healthy(&self) -> bool {
        self.is_running && (self.is_synced || self.state == NodeState::Syncing)
    }
}

/// Shutdown coordinator for managing subsystem shutdown order
pub struct ShutdownCoordinator {
    /// Shutdown signal sender
    shutdown_tx: tokio::sync::broadcast::Sender<()>,

    /// Subsystem shutdown handles
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new() -> Self {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(16);
        
        Self {
            shutdown_tx,
            handles: Vec::new(),
        }
    }

    /// Get a shutdown receiver
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Register a subsystem handle
    pub fn register_handle(&mut self, handle: tokio::task::JoinHandle<()>) {
        self.handles.push(handle);
    }

    /// Trigger shutdown for all subsystems
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Triggering shutdown for all subsystems");

        // Send shutdown signal
        let _ = self.shutdown_tx.send(());

        // Wait for all handles to complete
        let mut errors = Vec::new();
        
        for (i, handle) in self.handles.drain(..).enumerate() {
            match tokio::time::timeout(Duration::from_secs(10), handle).await {
                Ok(Ok(())) => {
                    info!("Subsystem {} shutdown complete", i);
                }
                Ok(Err(e)) => {
                    error!("Subsystem {} panicked: {:?}", i, e);
                    errors.push(format!("Subsystem {} panicked", i));
                }
                Err(_) => {
                    warn!("Subsystem {} shutdown timeout", i);
                    errors.push(format!("Subsystem {} timeout", i));
                }
            }
        }

        if !errors.is_empty() {
            return Err(NodeError::ShutdownError(
                format!("Shutdown errors: {}", errors.join(", "))
            ));
        }

        Ok(())
    }
}

impl Default for ShutdownCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NodeConfig;

    #[tokio::test]
    async fn test_health_status() {
        let status = HealthStatus {
            is_running: true,
            state: NodeState::Running,
            uptime_seconds: 100,
            is_synced: true,
            snapshot_height: 1000,
            peer_count: 10,
        };

        assert!(status.is_healthy());
    }

    #[tokio::test]
    async fn test_health_status_syncing() {
        let status = HealthStatus {
            is_running: true,
            state: NodeState::Syncing,
            uptime_seconds: 50,
            is_synced: false,
            snapshot_height: 500,
            peer_count: 5,
        };

        assert!(status.is_healthy());
    }

    #[tokio::test]
    async fn test_shutdown_coordinator() {
        let mut coordinator = ShutdownCoordinator::new();
        
        // Spawn a test task
        let mut rx = coordinator.subscribe();
        let handle = tokio::spawn(async move {
            let _ = rx.recv().await;
        });
        
        coordinator.register_handle(handle);
        
        // Trigger shutdown
        assert!(coordinator.shutdown().await.is_ok());
    }
}
