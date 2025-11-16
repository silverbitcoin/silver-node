//! # Node Configuration
//!
//! Configuration management for SilverBitcoin node.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Configuration error types
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Failed to load configuration file
    #[error("Failed to load config file: {0}")]
    LoadError(#[from] std::io::Error),

    /// Failed to parse configuration
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Invalid configuration value
    #[error("Invalid configuration: {0}")]
    ValidationError(String),

    /// Environment variable error
    #[error("Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),
}

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Complete node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Network configuration
    pub network: NetworkConfig,

    /// Consensus configuration
    pub consensus: ConsensusConfig,

    /// Storage configuration
    pub storage: StorageConfig,

    /// API configuration
    pub api: ApiConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Execution configuration
    pub execution: ExecutionConfig,

    /// GPU configuration
    #[serde(default)]
    pub gpu: GpuConfig,

    /// Memory management configuration (for consumer hardware)
    #[serde(default)]
    pub memory: MemoryManagementConfig,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network listening address
    pub listen_address: String,

    /// External address advertised to peers
    pub external_address: String,

    /// P2P communication address
    pub p2p_address: String,

    /// Maximum number of peer connections
    #[serde(default = "default_max_peers")]
    pub max_peers: usize,

    /// Seed nodes for initial peer discovery
    #[serde(default)]
    pub seed_nodes: Vec<String>,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Enable validator mode
    #[serde(default)]
    pub is_validator: bool,

    /// Path to validator key file
    pub validator_key_path: Option<PathBuf>,

    /// Validator stake amount
    pub stake_amount: Option<u64>,

    /// Snapshot interval in milliseconds
    #[serde(default = "default_snapshot_interval")]
    pub snapshot_interval_ms: u64,

    /// Maximum transactions per batch
    #[serde(default = "default_max_batch_transactions")]
    pub max_batch_transactions: usize,

    /// Maximum batch size in bytes
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size_bytes: usize,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database path
    pub db_path: PathBuf,

    /// Snapshot retention period in days
    #[serde(default = "default_snapshot_retention")]
    pub snapshot_retention_days: u32,

    /// Enable automatic pruning
    #[serde(default = "default_enable_pruning")]
    pub enable_pruning: bool,

    /// Object cache size in bytes
    #[serde(default = "default_cache_size")]
    pub object_cache_size: usize,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// JSON-RPC HTTP address
    pub json_rpc_address: String,

    /// WebSocket address for subscriptions
    pub websocket_address: String,

    /// Enable CORS
    #[serde(default = "default_enable_cors")]
    pub enable_cors: bool,

    /// Allowed CORS origins
    #[serde(default)]
    pub cors_origins: Vec<String>,

    /// Rate limit per client IP (requests per second)
    #[serde(default = "default_rate_limit")]
    pub rate_limit_per_second: u32,

    /// Maximum batch request size
    #[serde(default = "default_max_batch_size_api")]
    pub max_batch_size: usize,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Prometheus metrics endpoint
    pub prometheus_address: String,

    /// Enable metrics collection
    #[serde(default = "default_enable_metrics")]
    pub enable_metrics: bool,

    /// Metrics update interval in seconds
    #[serde(default = "default_update_interval")]
    pub update_interval_seconds: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log output path
    pub log_path: PathBuf,

    /// Enable JSON structured logging
    #[serde(default)]
    pub json_format: bool,

    /// Maximum log file size in MB
    #[serde(default = "default_max_log_size")]
    pub max_log_size_mb: u64,

    /// Maximum number of log files
    #[serde(default = "default_max_log_files")]
    pub max_log_files: usize,
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Number of worker threads for parallel execution
    #[serde(default = "default_worker_threads")]
    pub worker_threads: usize,

    /// Enable NUMA-aware memory allocation
    #[serde(default)]
    pub numa_aware: bool,

    /// Fuel price (MIST per fuel unit)
    #[serde(default = "default_fuel_price")]
    pub fuel_price: u64,
}

/// GPU configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuConfig {
    /// Enable GPU acceleration
    #[serde(default)]
    pub enable_gpu: bool,

    /// GPU backend: opencl, cuda, metal, auto
    #[serde(default = "default_gpu_backend")]
    pub backend: String,

    /// Minimum batch size for GPU execution
    #[serde(default = "default_min_batch_size")]
    pub min_batch_size: usize,
}

/// Memory management configuration (for consumer hardware optimization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryManagementConfig {
    /// Maximum memory usage in bytes (default: 8GB for 16GB systems)
    #[serde(default = "default_max_memory_usage")]
    pub max_memory_usage: u64,

    /// Enable memory monitoring
    #[serde(default = "default_enable_memory_monitoring")]
    pub enable_memory_monitoring: bool,

    /// Warning threshold (0.0 to 1.0)
    #[serde(default = "default_memory_warning_threshold")]
    pub memory_warning_threshold: f64,

    /// Critical threshold (0.0 to 1.0)
    #[serde(default = "default_memory_critical_threshold")]
    pub memory_critical_threshold: f64,

    /// Memory check interval in seconds
    #[serde(default = "default_memory_check_interval")]
    pub memory_check_interval: u64,
}

impl Default for MemoryManagementConfig {
    fn default() -> Self {
        Self {
            max_memory_usage: default_max_memory_usage(),
            enable_memory_monitoring: default_enable_memory_monitoring(),
            memory_warning_threshold: default_memory_warning_threshold(),
            memory_critical_threshold: default_memory_critical_threshold(),
            memory_check_interval: default_memory_check_interval(),
        }
    }
}

// Default value functions
fn default_max_peers() -> usize { 50 }
fn default_snapshot_interval() -> u64 { 480 }
fn default_max_batch_transactions() -> usize { 500 }
fn default_max_batch_size() -> usize { 524288 } // 512 KB
fn default_snapshot_retention() -> u32 { 30 }
fn default_enable_pruning() -> bool { true }
fn default_cache_size() -> usize { 1073741824 } // 1 GB
fn default_enable_cors() -> bool { true }
fn default_rate_limit() -> u32 { 100 }
fn default_max_batch_size_api() -> usize { 50 }
fn default_enable_metrics() -> bool { true }
fn default_update_interval() -> u64 { 1 }
fn default_log_level() -> String { "info".to_string() }
fn default_max_log_size() -> u64 { 100 }
fn default_max_log_files() -> usize { 10 }
fn default_worker_threads() -> usize { 16 }
fn default_fuel_price() -> u64 { 1000 }
fn default_gpu_backend() -> String { "auto".to_string() }
fn default_min_batch_size() -> usize { 100 }
fn default_max_memory_usage() -> u64 { 8 * 1024 * 1024 * 1024 } // 8 GB
fn default_enable_memory_monitoring() -> bool { true }
fn default_memory_warning_threshold() -> f64 { 0.85 }
fn default_memory_critical_threshold() -> f64 { 0.95 }
fn default_memory_check_interval() -> u64 { 30 }

impl NodeConfig {
    /// Load configuration from TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let mut config: NodeConfig = toml::from_str(&contents)?;
        
        // Apply environment variable overrides
        config.apply_env_overrides()?;
        
        // Validate configuration
        config.validate()?;
        
        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<()> {
        // Network overrides
        if let Ok(val) = std::env::var("SILVER_LISTEN_ADDRESS") {
            self.network.listen_address = val;
        }
        if let Ok(val) = std::env::var("SILVER_EXTERNAL_ADDRESS") {
            self.network.external_address = val;
        }
        if let Ok(val) = std::env::var("SILVER_P2P_ADDRESS") {
            self.network.p2p_address = val;
        }

        // Consensus overrides
        if let Ok(val) = std::env::var("SILVER_IS_VALIDATOR") {
            self.consensus.is_validator = val.parse().unwrap_or(false);
        }
        if let Ok(val) = std::env::var("SILVER_VALIDATOR_KEY_PATH") {
            self.consensus.validator_key_path = Some(PathBuf::from(val));
        }

        // Storage overrides
        if let Ok(val) = std::env::var("SILVER_DB_PATH") {
            self.storage.db_path = PathBuf::from(val);
        }

        // API overrides
        if let Ok(val) = std::env::var("SILVER_JSON_RPC_ADDRESS") {
            self.api.json_rpc_address = val;
        }
        if let Ok(val) = std::env::var("SILVER_WEBSOCKET_ADDRESS") {
            self.api.websocket_address = val;
        }

        // Logging overrides
        if let Ok(val) = std::env::var("SILVER_LOG_LEVEL") {
            self.logging.level = val;
        }

        Ok(())
    }

    /// Validate configuration values
    fn validate(&self) -> Result<()> {
        // Validate validator configuration
        if self.consensus.is_validator {
            if self.consensus.validator_key_path.is_none() {
                return Err(ConfigError::ValidationError(
                    "Validator mode requires validator_key_path".to_string()
                ));
            }
            if let Some(stake) = self.consensus.stake_amount {
                if stake < 1_000_000 {
                    return Err(ConfigError::ValidationError(
                        "Minimum validator stake is 1,000,000 SBTC".to_string()
                    ));
                }
            }
        }

        // Validate snapshot interval
        if self.consensus.snapshot_interval_ms == 0 {
            return Err(ConfigError::ValidationError(
                "Snapshot interval must be greater than 0".to_string()
            ));
        }

        // Validate batch limits
        if self.consensus.max_batch_transactions == 0 {
            return Err(ConfigError::ValidationError(
                "Max batch transactions must be greater than 0".to_string()
            ));
        }

        // Validate cache size
        if self.storage.object_cache_size < 1024 * 1024 {
            return Err(ConfigError::ValidationError(
                "Object cache size must be at least 1 MB".to_string()
            ));
        }

        // Validate rate limit
        if self.api.rate_limit_per_second == 0 {
            return Err(ConfigError::ValidationError(
                "Rate limit must be greater than 0".to_string()
            ));
        }

        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid log level: {}. Must be one of: trace, debug, info, warn, error", self.logging.level)
            ));
        }

        Ok(())
    }

    /// Create default configuration
    pub fn default() -> Self {
        Self {
            network: NetworkConfig {
                listen_address: "0.0.0.0:9000".to_string(),
                external_address: "127.0.0.1:9000".to_string(),
                p2p_address: "0.0.0.0:9001".to_string(),
                max_peers: default_max_peers(),
                seed_nodes: vec![],
            },
            consensus: ConsensusConfig {
                is_validator: false,
                validator_key_path: None,
                stake_amount: None,
                snapshot_interval_ms: default_snapshot_interval(),
                max_batch_transactions: default_max_batch_transactions(),
                max_batch_size_bytes: default_max_batch_size(),
            },
            storage: StorageConfig {
                db_path: PathBuf::from("./data/db"),
                snapshot_retention_days: default_snapshot_retention(),
                enable_pruning: default_enable_pruning(),
                object_cache_size: default_cache_size(),
            },
            api: ApiConfig {
                json_rpc_address: "0.0.0.0:9545".to_string(),
                websocket_address: "0.0.0.0:9546".to_string(),
                enable_cors: default_enable_cors(),
                cors_origins: vec![],
                rate_limit_per_second: default_rate_limit(),
                max_batch_size: default_max_batch_size_api(),
            },
            metrics: MetricsConfig {
                prometheus_address: "0.0.0.0:9184".to_string(),
                enable_metrics: default_enable_metrics(),
                update_interval_seconds: default_update_interval(),
            },
            logging: LoggingConfig {
                level: default_log_level(),
                log_path: PathBuf::from("./logs/node.log"),
                json_format: false,
                max_log_size_mb: default_max_log_size(),
                max_log_files: default_max_log_files(),
            },
            execution: ExecutionConfig {
                worker_threads: default_worker_threads(),
                numa_aware: false,
                fuel_price: default_fuel_price(),
            },
            gpu: GpuConfig::default(),
            memory: MemoryManagementConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NodeConfig::default();
        assert_eq!(config.network.max_peers, 50);
        assert_eq!(config.consensus.snapshot_interval_ms, 480);
        assert_eq!(config.storage.object_cache_size, 1073741824);
    }

    #[test]
    fn test_validator_validation() {
        let mut config = NodeConfig::default();
        config.consensus.is_validator = true;
        
        // Should fail without validator_key_path
        assert!(config.validate().is_err());
        
        // Should succeed with validator_key_path
        config.consensus.validator_key_path = Some(PathBuf::from("/tmp/key"));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_log_level() {
        let mut config = NodeConfig::default();
        config.logging.level = "invalid".to_string();
        assert!(config.validate().is_err());
    }
}
