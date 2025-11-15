//! # Metrics Collection and Export
//!
//! Prometheus-compatible metrics exporter for node monitoring.

use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry,
};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Metrics error types
#[derive(Error, Debug)]
pub enum MetricsError {
    /// Prometheus error
    #[error("Prometheus error: {0}")]
    PrometheusError(#[from] prometheus::Error),

    /// HTTP server error
    #[error("HTTP server error: {0}")]
    HttpError(String),

    /// Metrics not initialized
    #[error("Metrics not initialized")]
    NotInitialized,
}

/// Result type for metrics operations
pub type Result<T> = std::result::Result<T, MetricsError>;

/// Consensus metrics
#[derive(Clone)]
pub struct ConsensusMetrics {
    /// Total number of batches created
    pub batches_created: IntCounter,
    
    /// Total number of batches certified
    pub batches_certified: IntCounter,
    
    /// Total number of snapshots created
    pub snapshots_created: IntCounter,
    
    /// Current snapshot height
    pub snapshot_height: IntGauge,
    
    /// Consensus latency (time from batch creation to snapshot)
    pub consensus_latency_ms: Histogram,
    
    /// Batch size in transactions
    pub batch_size_transactions: Histogram,
    
    /// Batch size in bytes
    pub batch_size_bytes: Histogram,
    
    /// Number of active validators
    pub active_validators: IntGauge,
    
    /// Total stake weight
    pub total_stake: Gauge,
}

/// Execution metrics
#[derive(Clone)]
pub struct ExecutionMetrics {
    /// Total transactions executed
    pub transactions_executed: IntCounter,
    
    /// Total transactions failed
    pub transactions_failed: IntCounter,
    
    /// Transaction execution time
    pub execution_time_ms: Histogram,
    
    /// Fuel consumed
    pub fuel_consumed: Counter,
    
    /// Fuel refunded
    pub fuel_refunded: Counter,
    
    /// Parallel execution efficiency (0-1)
    pub parallel_efficiency: Gauge,
    
    /// Active execution threads
    pub active_threads: IntGauge,
}

/// Storage metrics
#[derive(Clone)]
pub struct StorageMetrics {
    /// Total objects stored
    pub objects_stored: IntGauge,
    
    /// Total transactions stored
    pub transactions_stored: IntGauge,
    
    /// Total events stored
    pub events_stored: IntGauge,
    
    /// Database size in bytes
    pub db_size_bytes: IntGauge,
    
    /// Cache hit rate (0-1)
    pub cache_hit_rate: Gauge,
    
    /// Read operations per second
    pub read_ops: IntCounter,
    
    /// Write operations per second
    pub write_ops: IntCounter,
    
    /// Read latency
    pub read_latency_ms: Histogram,
    
    /// Write latency
    pub write_latency_ms: Histogram,
}

/// Network metrics
#[derive(Clone)]
pub struct NetworkMetrics {
    /// Number of connected peers
    pub connected_peers: IntGauge,
    
    /// Total messages sent
    pub messages_sent: IntCounter,
    
    /// Total messages received
    pub messages_received: IntCounter,
    
    /// Total bytes sent
    pub bytes_sent: Counter,
    
    /// Total bytes received
    pub bytes_received: Counter,
    
    /// Message propagation latency
    pub propagation_latency_ms: Histogram,
    
    /// Peer reputation scores
    pub peer_reputation: Histogram,
    
    /// Blocked peers
    pub blocked_peers: IntGauge,
}

/// API metrics
#[derive(Clone)]
pub struct ApiMetrics {
    /// Total RPC requests
    pub rpc_requests: IntCounter,
    
    /// RPC requests by method
    pub rpc_requests_by_method: IntCounter,
    
    /// RPC request latency
    pub rpc_latency_ms: Histogram,
    
    /// Active WebSocket connections
    pub websocket_connections: IntGauge,
    
    /// Active subscriptions
    pub active_subscriptions: IntGauge,
    
    /// Rate limited requests
    pub rate_limited_requests: IntCounter,
}

/// System resource metrics
#[derive(Clone)]
pub struct SystemMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: Gauge,
    
    /// Memory usage in bytes
    pub memory_usage_bytes: IntGauge,
    
    /// Disk usage in bytes
    pub disk_usage_bytes: IntGauge,
    
    /// Disk available in bytes
    pub disk_available_bytes: IntGauge,
    
    /// Number of threads
    pub thread_count: IntGauge,
    
    /// File descriptors open
    pub file_descriptors: IntGauge,
}

/// Complete metrics collection
#[derive(Clone)]
pub struct NodeMetrics {
    pub consensus: ConsensusMetrics,
    pub execution: ExecutionMetrics,
    pub storage: StorageMetrics,
    pub network: NetworkMetrics,
    pub api: ApiMetrics,
    pub system: SystemMetrics,
}

/// Metrics exporter
pub struct MetricsExporter {
    /// Prometheus registry
    registry: Registry,
    
    /// Node metrics
    metrics: Arc<RwLock<Option<NodeMetrics>>>,
    
    /// Metrics server address
    address: String,
    
    /// Update interval in seconds
    update_interval: u64,
    
    /// Shutdown signal
    shutdown_rx: Option<tokio::sync::broadcast::Receiver<()>>,
}

impl MetricsExporter {
    /// Create a new metrics exporter
    pub fn new(
        address: String,
        update_interval: u64,
        shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<Self> {
        let registry = Registry::new();
        
        Ok(Self {
            registry,
            metrics: Arc::new(RwLock::new(None)),
            address,
            update_interval,
            shutdown_rx: Some(shutdown_rx),
        })
    }

    /// Initialize all metrics
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Prometheus metrics");

        // Create consensus metrics
        let consensus = ConsensusMetrics {
            batches_created: IntCounter::new(
                "silver_consensus_batches_created_total",
                "Total number of batches created"
            )?,
            batches_certified: IntCounter::new(
                "silver_consensus_batches_certified_total",
                "Total number of batches certified"
            )?,
            snapshots_created: IntCounter::new(
                "silver_consensus_snapshots_created_total",
                "Total number of snapshots created"
            )?,
            snapshot_height: IntGauge::new(
                "silver_consensus_snapshot_height",
                "Current snapshot height"
            )?,
            consensus_latency_ms: Histogram::with_opts(
                HistogramOpts::new(
                    "silver_consensus_latency_milliseconds",
                    "Consensus latency in milliseconds"
                ).buckets(vec![10.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0])
            )?,
            batch_size_transactions: Histogram::with_opts(
                HistogramOpts::new(
                    "silver_consensus_batch_size_transactions",
                    "Batch size in transactions"
                ).buckets(vec![10.0, 50.0, 100.0, 250.0, 500.0, 1000.0])
            )?,
            batch_size_bytes: Histogram::with_opts(
                HistogramOpts::new(
                    "silver_consensus_batch_size_bytes",
                    "Batch size in bytes"
                ).buckets(vec![1024.0, 10240.0, 102400.0, 524288.0, 1048576.0])
            )?,
            active_validators: IntGauge::new(
                "silver_consensus_active_validators",
                "Number of active validators"
            )?,
            total_stake: Gauge::new(
                "silver_consensus_total_stake",
                "Total stake weight"
            )?,
        };

        // Create execution metrics
        let execution = ExecutionMetrics {
            transactions_executed: IntCounter::new(
                "silver_execution_transactions_executed_total",
                "Total transactions executed"
            )?,
            transactions_failed: IntCounter::new(
                "silver_execution_transactions_failed_total",
                "Total transactions failed"
            )?,
            execution_time_ms: Histogram::with_opts(
                HistogramOpts::new(
                    "silver_execution_time_milliseconds",
                    "Transaction execution time in milliseconds"
                ).buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0, 500.0])
            )?,
            fuel_consumed: Counter::new(
                "silver_execution_fuel_consumed_total",
                "Total fuel consumed"
            )?,
            fuel_refunded: Counter::new(
                "silver_execution_fuel_refunded_total",
                "Total fuel refunded"
            )?,
            parallel_efficiency: Gauge::new(
                "silver_execution_parallel_efficiency",
                "Parallel execution efficiency (0-1)"
            )?,
            active_threads: IntGauge::new(
                "silver_execution_active_threads",
                "Active execution threads"
            )?,
        };

        // Create storage metrics
        let storage = StorageMetrics {
            objects_stored: IntGauge::new(
                "silver_storage_objects_total",
                "Total objects stored"
            )?,
            transactions_stored: IntGauge::new(
                "silver_storage_transactions_total",
                "Total transactions stored"
            )?,
            events_stored: IntGauge::new(
                "silver_storage_events_total",
                "Total events stored"
            )?,
            db_size_bytes: IntGauge::new(
                "silver_storage_db_size_bytes",
                "Database size in bytes"
            )?,
            cache_hit_rate: Gauge::new(
                "silver_storage_cache_hit_rate",
                "Cache hit rate (0-1)"
            )?,
            read_ops: IntCounter::new(
                "silver_storage_read_ops_total",
                "Total read operations"
            )?,
            write_ops: IntCounter::new(
                "silver_storage_write_ops_total",
                "Total write operations"
            )?,
            read_latency_ms: Histogram::with_opts(
                HistogramOpts::new(
                    "silver_storage_read_latency_milliseconds",
                    "Read latency in milliseconds"
                ).buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0])
            )?,
            write_latency_ms: Histogram::with_opts(
                HistogramOpts::new(
                    "silver_storage_write_latency_milliseconds",
                    "Write latency in milliseconds"
                ).buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 50.0, 100.0])
            )?,
        };

        // Create network metrics
        let network = NetworkMetrics {
            connected_peers: IntGauge::new(
                "silver_network_connected_peers",
                "Number of connected peers"
            )?,
            messages_sent: IntCounter::new(
                "silver_network_messages_sent_total",
                "Total messages sent"
            )?,
            messages_received: IntCounter::new(
                "silver_network_messages_received_total",
                "Total messages received"
            )?,
            bytes_sent: Counter::new(
                "silver_network_bytes_sent_total",
                "Total bytes sent"
            )?,
            bytes_received: Counter::new(
                "silver_network_bytes_received_total",
                "Total bytes received"
            )?,
            propagation_latency_ms: Histogram::with_opts(
                HistogramOpts::new(
                    "silver_network_propagation_latency_milliseconds",
                    "Message propagation latency in milliseconds"
                ).buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0])
            )?,
            peer_reputation: Histogram::with_opts(
                HistogramOpts::new(
                    "silver_network_peer_reputation",
                    "Peer reputation scores"
                ).buckets(vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0])
            )?,
            blocked_peers: IntGauge::new(
                "silver_network_blocked_peers",
                "Number of blocked peers"
            )?,
        };

        // Create API metrics
        let api = ApiMetrics {
            rpc_requests: IntCounter::new(
                "silver_api_rpc_requests_total",
                "Total RPC requests"
            )?,
            rpc_requests_by_method: IntCounter::new(
                "silver_api_rpc_requests_by_method_total",
                "RPC requests by method"
            )?,
            rpc_latency_ms: Histogram::with_opts(
                HistogramOpts::new(
                    "silver_api_rpc_latency_milliseconds",
                    "RPC request latency in milliseconds"
                ).buckets(vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0])
            )?,
            websocket_connections: IntGauge::new(
                "silver_api_websocket_connections",
                "Active WebSocket connections"
            )?,
            active_subscriptions: IntGauge::new(
                "silver_api_active_subscriptions",
                "Active event subscriptions"
            )?,
            rate_limited_requests: IntCounter::new(
                "silver_api_rate_limited_requests_total",
                "Rate limited requests"
            )?,
        };

        // Create system metrics
        let system = SystemMetrics {
            cpu_usage_percent: Gauge::new(
                "silver_system_cpu_usage_percent",
                "CPU usage percentage"
            )?,
            memory_usage_bytes: IntGauge::new(
                "silver_system_memory_usage_bytes",
                "Memory usage in bytes"
            )?,
            disk_usage_bytes: IntGauge::new(
                "silver_system_disk_usage_bytes",
                "Disk usage in bytes"
            )?,
            disk_available_bytes: IntGauge::new(
                "silver_system_disk_available_bytes",
                "Disk available in bytes"
            )?,
            thread_count: IntGauge::new(
                "silver_system_thread_count",
                "Number of threads"
            )?,
            file_descriptors: IntGauge::new(
                "silver_system_file_descriptors",
                "File descriptors open"
            )?,
        };

        // Register all metrics
        self.register_consensus_metrics(&consensus)?;
        self.register_execution_metrics(&execution)?;
        self.register_storage_metrics(&storage)?;
        self.register_network_metrics(&network)?;
        self.register_api_metrics(&api)?;
        self.register_system_metrics(&system)?;

        // Store metrics
        *self.metrics.write().await = Some(NodeMetrics {
            consensus,
            execution,
            storage,
            network,
            api,
            system,
        });

        info!("Metrics initialized successfully");
        Ok(())
    }

    /// Register consensus metrics
    fn register_consensus_metrics(&self, metrics: &ConsensusMetrics) -> Result<()> {
        self.registry.register(Box::new(metrics.batches_created.clone()))?;
        self.registry.register(Box::new(metrics.batches_certified.clone()))?;
        self.registry.register(Box::new(metrics.snapshots_created.clone()))?;
        self.registry.register(Box::new(metrics.snapshot_height.clone()))?;
        self.registry.register(Box::new(metrics.consensus_latency_ms.clone()))?;
        self.registry.register(Box::new(metrics.batch_size_transactions.clone()))?;
        self.registry.register(Box::new(metrics.batch_size_bytes.clone()))?;
        self.registry.register(Box::new(metrics.active_validators.clone()))?;
        self.registry.register(Box::new(metrics.total_stake.clone()))?;
        Ok(())
    }

    /// Register execution metrics
    fn register_execution_metrics(&self, metrics: &ExecutionMetrics) -> Result<()> {
        self.registry.register(Box::new(metrics.transactions_executed.clone()))?;
        self.registry.register(Box::new(metrics.transactions_failed.clone()))?;
        self.registry.register(Box::new(metrics.execution_time_ms.clone()))?;
        self.registry.register(Box::new(metrics.fuel_consumed.clone()))?;
        self.registry.register(Box::new(metrics.fuel_refunded.clone()))?;
        self.registry.register(Box::new(metrics.parallel_efficiency.clone()))?;
        self.registry.register(Box::new(metrics.active_threads.clone()))?;
        Ok(())
    }

    /// Register storage metrics
    fn register_storage_metrics(&self, metrics: &StorageMetrics) -> Result<()> {
        self.registry.register(Box::new(metrics.objects_stored.clone()))?;
        self.registry.register(Box::new(metrics.transactions_stored.clone()))?;
        self.registry.register(Box::new(metrics.events_stored.clone()))?;
        self.registry.register(Box::new(metrics.db_size_bytes.clone()))?;
        self.registry.register(Box::new(metrics.cache_hit_rate.clone()))?;
        self.registry.register(Box::new(metrics.read_ops.clone()))?;
        self.registry.register(Box::new(metrics.write_ops.clone()))?;
        self.registry.register(Box::new(metrics.read_latency_ms.clone()))?;
        self.registry.register(Box::new(metrics.write_latency_ms.clone()))?;
        Ok(())
    }

    /// Register network metrics
    fn register_network_metrics(&self, metrics: &NetworkMetrics) -> Result<()> {
        self.registry.register(Box::new(metrics.connected_peers.clone()))?;
        self.registry.register(Box::new(metrics.messages_sent.clone()))?;
        self.registry.register(Box::new(metrics.messages_received.clone()))?;
        self.registry.register(Box::new(metrics.bytes_sent.clone()))?;
        self.registry.register(Box::new(metrics.bytes_received.clone()))?;
        self.registry.register(Box::new(metrics.propagation_latency_ms.clone()))?;
        self.registry.register(Box::new(metrics.peer_reputation.clone()))?;
        self.registry.register(Box::new(metrics.blocked_peers.clone()))?;
        Ok(())
    }

    /// Register API metrics
    fn register_api_metrics(&self, metrics: &ApiMetrics) -> Result<()> {
        self.registry.register(Box::new(metrics.rpc_requests.clone()))?;
        self.registry.register(Box::new(metrics.rpc_requests_by_method.clone()))?;
        self.registry.register(Box::new(metrics.rpc_latency_ms.clone()))?;
        self.registry.register(Box::new(metrics.websocket_connections.clone()))?;
        self.registry.register(Box::new(metrics.active_subscriptions.clone()))?;
        self.registry.register(Box::new(metrics.rate_limited_requests.clone()))?;
        Ok(())
    }

    /// Register system metrics
    fn register_system_metrics(&self, metrics: &SystemMetrics) -> Result<()> {
        self.registry.register(Box::new(metrics.cpu_usage_percent.clone()))?;
        self.registry.register(Box::new(metrics.memory_usage_bytes.clone()))?;
        self.registry.register(Box::new(metrics.disk_usage_bytes.clone()))?;
        self.registry.register(Box::new(metrics.disk_available_bytes.clone()))?;
        self.registry.register(Box::new(metrics.thread_count.clone()))?;
        self.registry.register(Box::new(metrics.file_descriptors.clone()))?;
        Ok(())
    }

    /// Get metrics handle
    pub fn metrics(&self) -> Arc<RwLock<Option<NodeMetrics>>> {
        self.metrics.clone()
    }

    /// Start metrics HTTP server
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Prometheus metrics server on {}", self.address);

        let registry = self.registry.clone();
        let address = self.address.clone();
        let mut shutdown_rx = self.shutdown_rx.take()
            .ok_or(MetricsError::NotInitialized)?;

        // Spawn HTTP server
        tokio::spawn(async move {
            use axum::{
                routing::get,
                Router,
                response::IntoResponse,
                http::StatusCode,
            };
            use prometheus::TextEncoder;

            let app = Router::new()
                .route("/metrics", get(move || {
                    let registry = registry.clone();
                    async move {
                        let encoder = TextEncoder::new();
                        let metric_families = registry.gather();
                        match encoder.encode_to_string(&metric_families) {
                            Ok(metrics) => (StatusCode::OK, metrics).into_response(),
                            Err(e) => {
                                error!("Failed to encode metrics: {}", e);
                                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics").into_response()
                            }
                        }
                    }
                }));

            let listener = match tokio::net::TcpListener::bind(&address).await {
                Ok(listener) => listener,
                Err(e) => {
                    error!("Failed to bind metrics server: {}", e);
                    return;
                }
            };

            info!("Metrics server listening on {}", address);

            let server = axum::serve(listener, app);

            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        error!("Metrics server error: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    info!("Metrics server shutting down");
                }
            }
        });

        Ok(())
    }

    /// Start metrics update loop
    pub async fn start_update_loop(&self) {
        let metrics = self.metrics.clone();
        let update_interval = self.update_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(update_interval)
            );

            loop {
                interval.tick().await;

                // Update system metrics
                if let Some(metrics) = metrics.read().await.as_ref() {
                    Self::update_system_metrics(&metrics.system).await;
                }
            }
        });
    }

    /// Update system resource metrics
    async fn update_system_metrics(system: &SystemMetrics) {
        // Update CPU usage
        if let Ok(usage) = Self::get_cpu_usage() {
            system.cpu_usage_percent.set(usage);
        }

        // Update memory usage
        if let Ok(usage) = Self::get_memory_usage() {
            system.memory_usage_bytes.set(usage as i64);
        }

        // Update disk usage
        if let Ok((used, available)) = Self::get_disk_usage() {
            system.disk_usage_bytes.set(used as i64);
            system.disk_available_bytes.set(available as i64);
        }

        // Update thread count
        if let Ok(count) = Self::get_thread_count() {
            system.thread_count.set(count as i64);
        }

        // Update file descriptors
        if let Ok(count) = Self::get_file_descriptor_count() {
            system.file_descriptors.set(count as i64);
        }
    }

    /// Get CPU usage percentage
    fn get_cpu_usage() -> std::io::Result<f64> {
        // Use sysinfo or procfs to get CPU usage
        // For now, return a placeholder
        Ok(0.0)
    }

    /// Get memory usage in bytes
    fn get_memory_usage() -> std::io::Result<u64> {
        // Use sysinfo or procfs to get memory usage
        // For now, return a placeholder
        Ok(0)
    }

    /// Get disk usage (used, available) in bytes
    fn get_disk_usage() -> std::io::Result<(u64, u64)> {
        // Use statvfs or similar to get disk usage
        // For now, return placeholders
        Ok((0, 0))
    }

    /// Get thread count
    fn get_thread_count() -> std::io::Result<usize> {
        // Count threads in /proc/self/task or use sysinfo
        // For now, return a placeholder
        Ok(0)
    }

    /// Get file descriptor count
    fn get_file_descriptor_count() -> std::io::Result<usize> {
        // Count files in /proc/self/fd or use sysinfo
        // For now, return a placeholder
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_initialization() {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        let mut exporter = MetricsExporter::new(
            "127.0.0.1:9184".to_string(),
            1,
            rx,
        ).unwrap();

        assert!(exporter.initialize().await.is_ok());
        assert!(exporter.metrics.read().await.is_some());
    }

    #[tokio::test]
    async fn test_metrics_access() {
        let (tx, rx) = tokio::sync::broadcast::channel(1);
        let mut exporter = MetricsExporter::new(
            "127.0.0.1:9185".to_string(),
            1,
            rx,
        ).unwrap();

        exporter.initialize().await.unwrap();

        let metrics = exporter.metrics();
        let guard = metrics.read().await;
        let node_metrics = guard.as_ref().unwrap();

        // Test incrementing a counter
        node_metrics.consensus.batches_created.inc();
        assert_eq!(node_metrics.consensus.batches_created.get(), 1);
    }
}
