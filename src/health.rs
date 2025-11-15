//! # Health Check Endpoint
//!
//! HTTP health check endpoint for node monitoring and load balancing.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, error};

/// Health check error types
#[derive(Error, Debug)]
pub enum HealthError {
    /// HTTP server error
    #[error("HTTP server error: {0}")]
    HttpError(String),

    /// Health check not initialized
    #[error("Health check not initialized")]
    NotInitialized,
}

/// Result type for health check operations
pub type Result<T> = std::result::Result<T, HealthError>;

/// Node health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Node is healthy and operational
    Healthy,
    
    /// Node is syncing with network
    Syncing,
    
    /// Node is degraded but operational
    Degraded,
    
    /// Node is unhealthy
    Unhealthy,
}

/// Sync status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Is node synchronized with network
    pub is_synced: bool,
    
    /// Current snapshot height
    pub current_height: u64,
    
    /// Network snapshot height
    pub network_height: u64,
    
    /// Sync progress percentage (0-100)
    pub sync_progress: f64,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall health status
    pub status: HealthStatus,
    
    /// Sync status
    pub sync_status: SyncStatus,
    
    /// Number of connected peers
    pub peer_count: usize,
    
    /// Current snapshot height
    pub snapshot_height: u64,
    
    /// Uptime in seconds
    pub uptime_seconds: u64,
    
    /// Node version
    pub version: String,
    
    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Health check state
#[derive(Clone)]
pub struct HealthState {
    /// Current health status
    status: Arc<RwLock<HealthStatus>>,
    
    /// Sync status
    sync_status: Arc<RwLock<SyncStatus>>,
    
    /// Peer count
    peer_count: Arc<RwLock<usize>>,
    
    /// Snapshot height
    snapshot_height: Arc<RwLock<u64>>,
    
    /// Node start time
    start_time: std::time::Instant,
}

impl HealthState {
    /// Create a new health state
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(HealthStatus::Syncing)),
            sync_status: Arc::new(RwLock::new(SyncStatus {
                is_synced: false,
                current_height: 0,
                network_height: 0,
                sync_progress: 0.0,
            })),
            peer_count: Arc::new(RwLock::new(0)),
            snapshot_height: Arc::new(RwLock::new(0)),
            start_time: std::time::Instant::now(),
        }
    }

    /// Update health status
    pub async fn set_status(&self, status: HealthStatus) {
        *self.status.write().await = status;
    }

    /// Update sync status
    pub async fn set_sync_status(&self, sync_status: SyncStatus) {
        *self.sync_status.write().await = sync_status;
    }

    /// Update peer count
    pub async fn set_peer_count(&self, count: usize) {
        *self.peer_count.write().await = count;
    }

    /// Update snapshot height
    pub async fn set_snapshot_height(&self, height: u64) {
        *self.snapshot_height.write().await = height;
    }

    /// Get current health response
    pub async fn get_health(&self) -> HealthResponse {
        let status = *self.status.read().await;
        let sync_status = self.sync_status.read().await.clone();
        let peer_count = *self.peer_count.read().await;
        let snapshot_height = *self.snapshot_height.read().await;
        let uptime_seconds = self.start_time.elapsed().as_secs();

        HealthResponse {
            status,
            sync_status,
            peer_count,
            snapshot_height,
            uptime_seconds,
            version: env!("CARGO_PKG_VERSION").to_string(),
            details: None,
        }
    }

    /// Check if node is healthy
    pub async fn is_healthy(&self) -> bool {
        matches!(*self.status.read().await, HealthStatus::Healthy)
    }

    /// Check if node is synced
    pub async fn is_synced(&self) -> bool {
        self.sync_status.read().await.is_synced
    }
}

impl Default for HealthState {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check server
pub struct HealthCheckServer {
    /// Server address
    address: String,
    
    /// Health state
    state: HealthState,
    
    /// Shutdown signal
    shutdown_rx: Option<tokio::sync::broadcast::Receiver<()>>,
}

impl HealthCheckServer {
    /// Create a new health check server
    pub fn new(
        address: String,
        state: HealthState,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Self {
        Self {
            address,
            state,
            shutdown_rx: Some(shutdown_rx),
        }
    }

    /// Start health check HTTP server
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting health check server on {}", self.address);

        let state = self.state.clone();
        let address = self.address.clone();
        let mut shutdown_rx = self.shutdown_rx.take()
            .ok_or(HealthError::NotInitialized)?;

        tokio::spawn(async move {
            let app = Router::new()
                .route("/health", get(health_handler))
                .route("/ready", get(readiness_handler))
                .route("/live", get(liveness_handler))
                .with_state(state);

            let listener = match tokio::net::TcpListener::bind(&address).await {
                Ok(listener) => listener,
                Err(e) => {
                    error!("Failed to bind health check server: {}", e);
                    return;
                }
            };

            info!("Health check server listening on {}", address);

            let server = axum::serve(listener, app);

            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        error!("Health check server error: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Health check server shutting down");
                }
            }
        });

        Ok(())
    }

    /// Get health state
    pub fn state(&self) -> HealthState {
        self.state.clone()
    }
}

/// Health check handler
async fn health_handler(
    State(state): State<HealthState>,
) -> impl IntoResponse {
    let health = state.get_health().await;
    
    let status_code = match health.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Syncing => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK,
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(health))
}

/// Readiness check handler (for Kubernetes readiness probes)
async fn readiness_handler(
    State(state): State<HealthState>,
) -> impl IntoResponse {
    let is_synced = state.is_synced().await;
    let peer_count = *state.peer_count.read().await;

    if is_synced && peer_count > 0 {
        (StatusCode::OK, "Ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Not ready")
    }
}

/// Liveness check handler (for Kubernetes liveness probes)
async fn liveness_handler(
    State(state): State<HealthState>,
) -> impl IntoResponse {
    let is_healthy = state.is_healthy().await;

    if is_healthy || matches!(*state.status.read().await, HealthStatus::Syncing | HealthStatus::Degraded) {
        (StatusCode::OK, "Alive")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Not alive")
    }
}

/// Health monitor that updates health status based on node state
pub struct HealthMonitor {
    /// Health state
    state: HealthState,
    
    /// Update interval in seconds
    update_interval: u64,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(state: HealthState, update_interval: u64) -> Self {
        Self {
            state,
            update_interval,
        }
    }

    /// Start health monitoring loop
    pub async fn start(&self) {
        let state = self.state.clone();
        let update_interval = self.update_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(update_interval)
            );

            loop {
                interval.tick().await;
                Self::update_health_status(&state).await;
            }
        });
    }

    /// Update health status based on node state
    async fn update_health_status(state: &HealthState) {
        // Get current state
        let sync_status = state.sync_status.read().await.clone();
        let peer_count = *state.peer_count.read().await;

        // Determine health status
        let health_status = if sync_status.is_synced && peer_count >= 3 {
            HealthStatus::Healthy
        } else if sync_status.is_synced && peer_count > 0 {
            HealthStatus::Degraded
        } else if !sync_status.is_synced {
            HealthStatus::Syncing
        } else {
            HealthStatus::Unhealthy
        };

        state.set_status(health_status).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_state_creation() {
        let state = HealthState::new();
        assert_eq!(*state.status.read().await, HealthStatus::Syncing);
        assert!(!state.is_healthy().await);
        assert!(!state.is_synced().await);
    }

    #[tokio::test]
    async fn test_health_state_updates() {
        let state = HealthState::new();

        state.set_status(HealthStatus::Healthy).await;
        assert!(state.is_healthy().await);

        state.set_peer_count(5).await;
        assert_eq!(*state.peer_count.read().await, 5);

        state.set_snapshot_height(100).await;
        assert_eq!(*state.snapshot_height.read().await, 100);
    }

    #[tokio::test]
    async fn test_health_response() {
        let state = HealthState::new();
        
        state.set_status(HealthStatus::Healthy).await;
        state.set_peer_count(10).await;
        state.set_snapshot_height(1000).await;
        state.set_sync_status(SyncStatus {
            is_synced: true,
            current_height: 1000,
            network_height: 1000,
            sync_progress: 100.0,
        }).await;

        let health = state.get_health().await;
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.peer_count, 10);
        assert_eq!(health.snapshot_height, 1000);
        assert!(health.sync_status.is_synced);
    }

    #[tokio::test]
    async fn test_health_monitor_status_determination() {
        let state = HealthState::new();

        // Test syncing state
        state.set_sync_status(SyncStatus {
            is_synced: false,
            current_height: 50,
            network_height: 100,
            sync_progress: 50.0,
        }).await;
        state.set_peer_count(5).await;
        HealthMonitor::update_health_status(&state).await;
        assert_eq!(*state.status.read().await, HealthStatus::Syncing);

        // Test healthy state
        state.set_sync_status(SyncStatus {
            is_synced: true,
            current_height: 100,
            network_height: 100,
            sync_progress: 100.0,
        }).await;
        state.set_peer_count(5).await;
        HealthMonitor::update_health_status(&state).await;
        assert_eq!(*state.status.read().await, HealthStatus::Healthy);

        // Test degraded state (low peer count)
        state.set_peer_count(1).await;
        HealthMonitor::update_health_status(&state).await;
        assert_eq!(*state.status.read().await, HealthStatus::Degraded);

        // Test unhealthy state (no peers)
        state.set_peer_count(0).await;
        HealthMonitor::update_health_status(&state).await;
        assert_eq!(*state.status.read().await, HealthStatus::Unhealthy);
    }
}
