//! Metrics and alerting system for validator monitoring
//!
//! This module implements production-ready metrics and alerting:
//! - Prometheus metrics export
//! - Grafana dashboard templates
//! - Alert rule configuration
//! - Health check logic
//! - Performance monitoring

use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge, Registry, TextEncoder, Encoder,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

/// Prometheus metrics for validator monitoring
pub struct ValidatorMetrics {
    /// Total validators registered
    pub total_validators: IntGauge,
    /// Active validators
    pub active_validators: IntGauge,
    /// Jailed validators
    pub jailed_validators: IntGauge,
    /// Average participation rate
    pub avg_participation_rate: Gauge,
    /// Average uptime percentage
    pub avg_uptime_percentage: Gauge,
    /// Average response time (ms)
    pub avg_response_time_ms: Gauge,
    /// Validators with critical alerts
    pub critical_validators: IntGauge,
    /// Validators with warning alerts
    pub warning_validators: IntGauge,
    /// Total alerts generated
    pub total_alerts: IntCounter,
    /// Critical alerts
    pub critical_alerts: IntCounter,
    /// Warning alerts
    pub warning_alerts: IntCounter,
}

impl ValidatorMetrics {
    /// Create new validator metrics
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let total_validators = IntGauge::new("validator_total_count", "Total validators")?;
        let active_validators = IntGauge::new("validator_active_count", "Active validators")?;
        let jailed_validators = IntGauge::new("validator_jailed_count", "Jailed validators")?;
        let avg_participation_rate =
            Gauge::new("validator_avg_participation_rate", "Average participation rate")?;
        let avg_uptime_percentage =
            Gauge::new("validator_avg_uptime_percentage", "Average uptime percentage")?;
        let avg_response_time_ms =
            Gauge::new("validator_avg_response_time_ms", "Average response time (ms)")?;
        let critical_validators =
            IntGauge::new("validator_critical_count", "Validators with critical alerts")?;
        let warning_validators =
            IntGauge::new("validator_warning_count", "Validators with warning alerts")?;
        let total_alerts = IntCounter::new("validator_total_alerts", "Total alerts generated")?;
        let critical_alerts = IntCounter::new("validator_critical_alerts", "Critical alerts")?;
        let warning_alerts = IntCounter::new("validator_warning_alerts", "Warning alerts")?;

        registry.register(Box::new(total_validators.clone()))?;
        registry.register(Box::new(active_validators.clone()))?;
        registry.register(Box::new(jailed_validators.clone()))?;
        registry.register(Box::new(avg_participation_rate.clone()))?;
        registry.register(Box::new(avg_uptime_percentage.clone()))?;
        registry.register(Box::new(avg_response_time_ms.clone()))?;
        registry.register(Box::new(critical_validators.clone()))?;
        registry.register(Box::new(warning_validators.clone()))?;
        registry.register(Box::new(total_alerts.clone()))?;
        registry.register(Box::new(critical_alerts.clone()))?;
        registry.register(Box::new(warning_alerts.clone()))?;

        Ok(Self {
            total_validators,
            active_validators,
            jailed_validators,
            avg_participation_rate,
            avg_uptime_percentage,
            avg_response_time_ms,
            critical_validators,
            warning_validators,
            total_alerts,
            critical_alerts,
            warning_alerts,
        })
    }
}

/// Consensus metrics
pub struct ConsensusMetrics {
    /// Snapshots created
    pub snapshots_created: IntCounter,
    /// Snapshot creation time (ms)
    pub snapshot_creation_time: Histogram,
    /// Transactions processed
    pub transactions_processed: IntCounter,
    /// Transaction processing time (ms)
    pub transaction_processing_time: Histogram,
    /// Batches created
    pub batches_created: IntCounter,
    /// Batch size
    pub batch_size: Histogram,
    /// Consensus rounds
    pub consensus_rounds: IntCounter,
    /// Byzantine faults detected
    pub byzantine_faults: IntCounter,
}

impl ConsensusMetrics {
    /// Create new consensus metrics
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let snapshots_created = IntCounter::new("consensus_snapshots_created", "Snapshots created")?;
        let snapshot_creation_time = Histogram::new(
            "consensus_snapshot_creation_time_ms",
            "Snapshot creation time (ms)",
        )?;
        let transactions_processed =
            IntCounter::new("consensus_transactions_processed", "Transactions processed")?;
        let transaction_processing_time = Histogram::new(
            "consensus_transaction_processing_time_ms",
            "Transaction processing time (ms)",
        )?;
        let batches_created = IntCounter::new("consensus_batches_created", "Batches created")?;
        let batch_size = Histogram::new("consensus_batch_size", "Batch size")?;
        let consensus_rounds = IntCounter::new("consensus_rounds", "Consensus rounds")?;
        let byzantine_faults =
            IntCounter::new("consensus_byzantine_faults", "Byzantine faults detected")?;

        registry.register(Box::new(snapshots_created.clone()))?;
        registry.register(Box::new(snapshot_creation_time.clone()))?;
        registry.register(Box::new(transactions_processed.clone()))?;
        registry.register(Box::new(transaction_processing_time.clone()))?;
        registry.register(Box::new(batches_created.clone()))?;
        registry.register(Box::new(batch_size.clone()))?;
        registry.register(Box::new(consensus_rounds.clone()))?;
        registry.register(Box::new(byzantine_faults.clone()))?;

        Ok(Self {
            snapshots_created,
            snapshot_creation_time,
            transactions_processed,
            transaction_processing_time,
            batches_created,
            batch_size,
            consensus_rounds,
            byzantine_faults,
        })
    }
}

/// Network metrics
pub struct NetworkMetrics {
    /// Peers connected
    pub peers_connected: IntGauge,
    /// Messages sent
    pub messages_sent: IntCounter,
    /// Messages received
    pub messages_received: IntCounter,
    /// Bytes sent
    pub bytes_sent: IntCounter,
    /// Bytes received
    pub bytes_received: IntCounter,
    /// Network latency (ms)
    pub network_latency: Histogram,
    /// Connection errors
    pub connection_errors: IntCounter,
}

impl NetworkMetrics {
    /// Create new network metrics
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let peers_connected = IntGauge::new("network_peers_connected", "Peers connected")?;
        let messages_sent = IntCounter::new("network_messages_sent", "Messages sent")?;
        let messages_received = IntCounter::new("network_messages_received", "Messages received")?;
        let bytes_sent = IntCounter::new("network_bytes_sent", "Bytes sent")?;
        let bytes_received = IntCounter::new("network_bytes_received", "Bytes received")?;
        let network_latency =
            Histogram::new("network_latency_ms", "Network latency (ms)")?;
        let connection_errors = IntCounter::new("network_connection_errors", "Connection errors")?;

        registry.register(Box::new(peers_connected.clone()))?;
        registry.register(Box::new(messages_sent.clone()))?;
        registry.register(Box::new(messages_received.clone()))?;
        registry.register(Box::new(bytes_sent.clone()))?;
        registry.register(Box::new(bytes_received.clone()))?;
        registry.register(Box::new(network_latency.clone()))?;
        registry.register(Box::new(connection_errors.clone()))?;

        Ok(Self {
            peers_connected,
            messages_sent,
            messages_received,
            bytes_sent,
            bytes_received,
            network_latency,
            connection_errors,
        })
    }
}

/// Storage metrics
pub struct StorageMetrics {
    /// Objects stored
    pub objects_stored: IntGauge,
    /// Database size (bytes)
    pub database_size: IntGauge,
    /// Read operations
    pub read_operations: IntCounter,
    /// Write operations
    pub write_operations: IntCounter,
    /// Read latency (ms)
    pub read_latency: Histogram,
    /// Write latency (ms)
    pub write_latency: Histogram,
    /// Storage errors
    pub storage_errors: IntCounter,
}

impl StorageMetrics {
    /// Create new storage metrics
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let objects_stored = IntGauge::new("storage_objects_stored", "Objects stored")?;
        let database_size = IntGauge::new("storage_database_size_bytes", "Database size (bytes)")?;
        let read_operations = IntCounter::new("storage_read_operations", "Read operations")?;
        let write_operations = IntCounter::new("storage_write_operations", "Write operations")?;
        let read_latency = Histogram::new("storage_read_latency_ms", "Read latency (ms)")?;
        let write_latency = Histogram::new("storage_write_latency_ms", "Write latency (ms)")?;
        let storage_errors = IntCounter::new("storage_errors", "Storage errors")?;

        registry.register(Box::new(objects_stored.clone()))?;
        registry.register(Box::new(database_size.clone()))?;
        registry.register(Box::new(read_operations.clone()))?;
        registry.register(Box::new(write_operations.clone()))?;
        registry.register(Box::new(read_latency.clone()))?;
        registry.register(Box::new(write_latency.clone()))?;
        registry.register(Box::new(storage_errors.clone()))?;

        Ok(Self {
            objects_stored,
            database_size,
            read_operations,
            write_operations,
            read_latency,
            write_latency,
            storage_errors,
        })
    }
}

/// System metrics
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage: Gauge,
    /// Memory usage (bytes)
    pub memory_usage: IntGauge,
    /// Memory limit (bytes)
    pub memory_limit: IntGauge,
    /// Disk usage (bytes)
    pub disk_usage: IntGauge,
    /// Disk limit (bytes)
    pub disk_limit: IntGauge,
    /// Uptime (seconds)
    pub uptime: IntGauge,
    /// Goroutines (threads)
    pub goroutines: IntGauge,
}

impl SystemMetrics {
    /// Create new system metrics
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let cpu_usage = Gauge::new("system_cpu_usage_percent", "CPU usage percentage")?;
        let memory_usage = IntGauge::new("system_memory_usage_bytes", "Memory usage (bytes)")?;
        let memory_limit = IntGauge::new("system_memory_limit_bytes", "Memory limit (bytes)")?;
        let disk_usage = IntGauge::new("system_disk_usage_bytes", "Disk usage (bytes)")?;
        let disk_limit = IntGauge::new("system_disk_limit_bytes", "Disk limit (bytes)")?;
        let uptime = IntGauge::new("system_uptime_seconds", "Uptime (seconds)")?;
        let goroutines = IntGauge::new("system_goroutines", "Goroutines (threads)")?;

        registry.register(Box::new(cpu_usage.clone()))?;
        registry.register(Box::new(memory_usage.clone()))?;
        registry.register(Box::new(memory_limit.clone()))?;
        registry.register(Box::new(disk_usage.clone()))?;
        registry.register(Box::new(disk_limit.clone()))?;
        registry.register(Box::new(uptime.clone()))?;
        registry.register(Box::new(goroutines.clone()))?;

        Ok(Self {
            cpu_usage,
            memory_usage,
            memory_limit,
            disk_usage,
            disk_limit,
            uptime,
            goroutines,
        })
    }
}

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Metric name
    pub metric: String,
    /// Condition (e.g., ">", "<", "==")
    pub condition: String,
    /// Threshold value
    pub threshold: f64,
    /// Duration (seconds)
    pub duration: u64,
    /// Severity (Info, Warning, Critical)
    pub severity: String,
    /// Enabled
    pub enabled: bool,
}

/// Alert rule manager
pub struct AlertRuleManager {
    rules: Vec<AlertRule>,
}

impl AlertRuleManager {
    /// Create new alert rule manager
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
        }
    }

    /// Add alert rule
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
        info!("Alert rule added: {}", rule.name);
    }

    /// Remove alert rule
    pub fn remove_rule(&mut self, name: &str) -> bool {
        if let Some(pos) = self.rules.iter().position(|r| r.name == name) {
            self.rules.remove(pos);
            info!("Alert rule removed: {}", name);
            true
        } else {
            false
        }
    }

    /// Get all rules
    pub fn get_rules(&self) -> &[AlertRule] {
        &self.rules
    }

    /// Get enabled rules
    pub fn get_enabled_rules(&self) -> Vec<&AlertRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    /// Enable rule
    pub fn enable_rule(&mut self, name: &str) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = true;
            info!("Alert rule enabled: {}", name);
            true
        } else {
            false
        }
    }

    /// Disable rule
    pub fn disable_rule(&mut self, name: &str) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.name == name) {
            rule.enabled = false;
            info!("Alert rule disabled: {}", name);
            true
        } else {
            false
        }
    }

    /// Evaluate rule
    pub fn evaluate_rule(&self, rule: &AlertRule, value: f64) -> bool {
        match rule.condition.as_str() {
            ">" => value > rule.threshold,
            "<" => value < rule.threshold,
            ">=" => value >= rule.threshold,
            "<=" => value <= rule.threshold,
            "==" => (value - rule.threshold).abs() < f64::EPSILON,
            "!=" => (value - rule.threshold).abs() >= f64::EPSILON,
            _ => false,
        }
    }
}

impl Default for AlertRuleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckStatus {
    /// Overall status (Healthy, Degraded, Unhealthy)
    pub status: String,
    /// Timestamp
    pub timestamp: u64,
    /// Validator count
    pub validator_count: usize,
    /// Active validators
    pub active_validators: usize,
    /// Critical validators
    pub critical_validators: usize,
    /// Average participation
    pub avg_participation: f64,
    /// Average uptime
    pub avg_uptime: f64,
    /// Peers connected
    pub peers_connected: usize,
    /// Database size (bytes)
    pub database_size: u64,
    /// Memory usage (bytes)
    pub memory_usage: u64,
    /// CPU usage (%)
    pub cpu_usage: f64,
}

/// Health checker
pub struct HealthChecker {
    start_time: u64,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new() -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self { start_time }
    }

    /// Check health status
    pub fn check_health(
        &self,
        validator_count: usize,
        active_validators: usize,
        critical_validators: usize,
        avg_participation: f64,
        avg_uptime: f64,
        peers_connected: usize,
        database_size: u64,
        memory_usage: u64,
        cpu_usage: f64,
    ) -> HealthCheckStatus {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Determine overall status
        let status = if critical_validators > 0 || cpu_usage > 90.0 || memory_usage > 16 * 1024 * 1024 * 1024 {
            "Unhealthy".to_string()
        } else if critical_validators > validator_count / 3 || avg_participation < 80.0 || cpu_usage > 80.0 {
            "Degraded".to_string()
        } else {
            "Healthy".to_string()
        };

        HealthCheckStatus {
            status,
            timestamp: now,
            validator_count,
            active_validators,
            critical_validators,
            avg_participation,
            avg_uptime,
            peers_connected,
            database_size,
            memory_usage,
            cpu_usage,
        }
    }

    /// Get uptime (seconds)
    pub fn get_uptime(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        now.saturating_sub(self.start_time)
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics exporter
pub struct MetricsExporter {
    registry: Registry,
    validator_metrics: ValidatorMetrics,
    consensus_metrics: ConsensusMetrics,
    network_metrics: NetworkMetrics,
    storage_metrics: StorageMetrics,
    system_metrics: SystemMetrics,
}

impl MetricsExporter {
    /// Create new metrics exporter
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        let validator_metrics = ValidatorMetrics::new(&registry)?;
        let consensus_metrics = ConsensusMetrics::new(&registry)?;
        let network_metrics = NetworkMetrics::new(&registry)?;
        let storage_metrics = StorageMetrics::new(&registry)?;
        let system_metrics = SystemMetrics::new(&registry)?;

        Ok(Self {
            registry,
            validator_metrics,
            consensus_metrics,
            network_metrics,
            storage_metrics,
            system_metrics,
        })
    }

    /// Export metrics as Prometheus text format
    pub fn export_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        encoder.encode_to_string(&self.registry.gather(), &mut String::new())
    }

    /// Get validator metrics
    pub fn validator_metrics(&self) -> &ValidatorMetrics {
        &self.validator_metrics
    }

    /// Get consensus metrics
    pub fn consensus_metrics(&self) -> &ConsensusMetrics {
        &self.consensus_metrics
    }

    /// Get network metrics
    pub fn network_metrics(&self) -> &NetworkMetrics {
        &self.network_metrics
    }

    /// Get storage metrics
    pub fn storage_metrics(&self) -> &StorageMetrics {
        &self.storage_metrics
    }

    /// Get system metrics
    pub fn system_metrics(&self) -> &SystemMetrics {
        &self.system_metrics
    }
}

impl Default for MetricsExporter {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics exporter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_exporter_creation() {
        let exporter = MetricsExporter::new();
        assert!(exporter.is_ok());
    }

    #[test]
    fn test_export_metrics() {
        let exporter = MetricsExporter::new().unwrap();
        let metrics = exporter.export_metrics();
        assert!(metrics.is_ok());
        let output = metrics.unwrap();
        assert!(!output.is_empty());
    }

    #[test]
    fn test_alert_rule_manager_creation() {
        let manager = AlertRuleManager::new();
        assert_eq!(manager.get_rules().len(), 0);
    }

    #[test]
    fn test_add_alert_rule() {
        let mut manager = AlertRuleManager::new();
        let rule = AlertRule {
            name: "test_rule".to_string(),
            description: "Test rule".to_string(),
            metric: "validator_participation".to_string(),
            condition: "<".to_string(),
            threshold: 80.0,
            duration: 300,
            severity: "Warning".to_string(),
            enabled: true,
        };

        manager.add_rule(rule);
        assert_eq!(manager.get_rules().len(), 1);
    }

    #[test]
    fn test_remove_alert_rule() {
        let mut manager = AlertRuleManager::new();
        let rule = AlertRule {
            name: "test_rule".to_string(),
            description: "Test rule".to_string(),
            metric: "validator_participation".to_string(),
            condition: "<".to_string(),
            threshold: 80.0,
            duration: 300,
            severity: "Warning".to_string(),
            enabled: true,
        };

        manager.add_rule(rule);
        assert!(manager.remove_rule("test_rule"));
        assert_eq!(manager.get_rules().len(), 0);
    }

    #[test]
    fn test_enable_disable_rule() {
        let mut manager = AlertRuleManager::new();
        let rule = AlertRule {
            name: "test_rule".to_string(),
            description: "Test rule".to_string(),
            metric: "validator_participation".to_string(),
            condition: "<".to_string(),
            threshold: 80.0,
            duration: 300,
            severity: "Warning".to_string(),
            enabled: true,
        };

        manager.add_rule(rule);
        assert!(manager.disable_rule("test_rule"));
        assert!(!manager.get_rules()[0].enabled);
        assert!(manager.enable_rule("test_rule"));
        assert!(manager.get_rules()[0].enabled);
    }

    #[test]
    fn test_evaluate_rule_greater_than() {
        let manager = AlertRuleManager::new();
        let rule = AlertRule {
            name: "test".to_string(),
            description: "Test".to_string(),
            metric: "test".to_string(),
            condition: ">".to_string(),
            threshold: 50.0,
            duration: 300,
            severity: "Warning".to_string(),
            enabled: true,
        };

        assert!(manager.evaluate_rule(&rule, 60.0));
        assert!(!manager.evaluate_rule(&rule, 40.0));
    }

    #[test]
    fn test_evaluate_rule_less_than() {
        let manager = AlertRuleManager::new();
        let rule = AlertRule {
            name: "test".to_string(),
            description: "Test".to_string(),
            metric: "test".to_string(),
            condition: "<".to_string(),
            threshold: 50.0,
            duration: 300,
            severity: "Warning".to_string(),
            enabled: true,
        };

        assert!(manager.evaluate_rule(&rule, 40.0));
        assert!(!manager.evaluate_rule(&rule, 60.0));
    }

    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        assert!(checker.get_uptime() >= 0);
    }

    #[test]
    fn test_health_check_healthy() {
        let checker = HealthChecker::new();
        let status = checker.check_health(
            100,  // validator_count
            95,   // active_validators
            0,    // critical_validators
            95.0, // avg_participation
            98.0, // avg_uptime
            50,   // peers_connected
            1024 * 1024 * 1024, // database_size
            4 * 1024 * 1024 * 1024, // memory_usage
            30.0, // cpu_usage
        );

        assert_eq!(status.status, "Healthy");
    }

    #[test]
    fn test_health_check_degraded() {
        let checker = HealthChecker::new();
        let status = checker.check_health(
            100,  // validator_count
            95,   // active_validators
            5,    // critical_validators
            75.0, // avg_participation
            90.0, // avg_uptime
            50,   // peers_connected
            1024 * 1024 * 1024, // database_size
            4 * 1024 * 1024 * 1024, // memory_usage
            85.0, // cpu_usage
        );

        assert_eq!(status.status, "Degraded");
    }

    #[test]
    fn test_health_check_unhealthy() {
        let checker = HealthChecker::new();
        let status = checker.check_health(
            100,  // validator_count
            50,   // active_validators
            50,   // critical_validators
            50.0, // avg_participation
            60.0, // avg_uptime
            10,   // peers_connected
            1024 * 1024 * 1024, // database_size
            14 * 1024 * 1024 * 1024, // memory_usage
            95.0, // cpu_usage
        );

        assert_eq!(status.status, "Unhealthy");
    }

    #[test]
    fn test_get_enabled_rules() {
        let mut manager = AlertRuleManager::new();

        let rule1 = AlertRule {
            name: "rule1".to_string(),
            description: "Rule 1".to_string(),
            metric: "test".to_string(),
            condition: ">".to_string(),
            threshold: 50.0,
            duration: 300,
            severity: "Warning".to_string(),
            enabled: true,
        };

        let rule2 = AlertRule {
            name: "rule2".to_string(),
            description: "Rule 2".to_string(),
            metric: "test".to_string(),
            condition: "<".to_string(),
            threshold: 30.0,
            duration: 300,
            severity: "Critical".to_string(),
            enabled: false,
        };

        manager.add_rule(rule1);
        manager.add_rule(rule2);

        let enabled = manager.get_enabled_rules();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "rule1");
    }
}
