use silver_node::metrics_alerting::{
    MetricsExporter, AlertRuleManager, AlertRule, HealthChecker, HealthCheckStatus,
};

#[test]
fn test_metrics_exporter_creation() {
    let exporter = MetricsExporter::new();
    assert!(exporter.is_ok());
}

#[test]
fn test_export_metrics_format() {
    let exporter = MetricsExporter::new().unwrap();
    let metrics = exporter.export_metrics();
    assert!(metrics.is_ok());

    let output = metrics.unwrap();
    assert!(!output.is_empty());
    // Prometheus format should contain TYPE and HELP comments
    assert!(output.contains("# HELP") || output.contains("# TYPE"));
}

#[test]
fn test_validator_metrics() {
    let exporter = MetricsExporter::new().unwrap();
    let validator_metrics = exporter.validator_metrics();

    // Set some metrics
    validator_metrics.total_validators.set(100);
    validator_metrics.active_validators.set(95);
    validator_metrics.jailed_validators.set(5);
    validator_metrics.avg_participation_rate.set(97.5);
    validator_metrics.avg_uptime_percentage.set(98.2);
    validator_metrics.avg_response_time_ms.set(150.0);
    validator_metrics.critical_validators.set(0);
    validator_metrics.warning_validators.set(3);

    let metrics = exporter.export_metrics().unwrap();
    assert!(metrics.contains("validator_total_count"));
    assert!(metrics.contains("validator_active_count"));
}

#[test]
fn test_consensus_metrics() {
    let exporter = MetricsExporter::new().unwrap();
    let consensus_metrics = exporter.consensus_metrics();

    consensus_metrics.snapshots_created.inc();
    consensus_metrics.snapshots_created.inc();
    consensus_metrics.transactions_processed.inc_by(100);
    consensus_metrics.batches_created.inc();

    let metrics = exporter.export_metrics().unwrap();
    assert!(metrics.contains("consensus_snapshots_created"));
    assert!(metrics.contains("consensus_transactions_processed"));
}

#[test]
fn test_network_metrics() {
    let exporter = MetricsExporter::new().unwrap();
    let network_metrics = exporter.network_metrics();

    network_metrics.peers_connected.set(50);
    network_metrics.messages_sent.inc_by(1000);
    network_metrics.messages_received.inc_by(1000);
    network_metrics.bytes_sent.inc_by(1024 * 1024);
    network_metrics.bytes_received.inc_by(1024 * 1024);

    let metrics = exporter.export_metrics().unwrap();
    assert!(metrics.contains("network_peers_connected"));
    assert!(metrics.contains("network_messages_sent"));
}

#[test]
fn test_storage_metrics() {
    let exporter = MetricsExporter::new().unwrap();
    let storage_metrics = exporter.storage_metrics();

    storage_metrics.objects_stored.set(1000000);
    storage_metrics.database_size.set(10 * 1024 * 1024 * 1024);
    storage_metrics.read_operations.inc_by(10000);
    storage_metrics.write_operations.inc_by(5000);

    let metrics = exporter.export_metrics().unwrap();
    assert!(metrics.contains("storage_objects_stored"));
    assert!(metrics.contains("storage_database_size_bytes"));
}

#[test]
fn test_system_metrics() {
    let exporter = MetricsExporter::new().unwrap();
    let system_metrics = exporter.system_metrics();

    system_metrics.cpu_usage.set(45.5);
    system_metrics.memory_usage.set(8 * 1024 * 1024 * 1024);
    system_metrics.memory_limit.set(16 * 1024 * 1024 * 1024);
    system_metrics.disk_usage.set(500 * 1024 * 1024 * 1024);
    system_metrics.disk_limit.set(1024 * 1024 * 1024 * 1024);
    system_metrics.uptime.set(86400);
    system_metrics.goroutines.set(100);

    let metrics = exporter.export_metrics().unwrap();
    assert!(metrics.contains("system_cpu_usage_percent"));
    assert!(metrics.contains("system_memory_usage_bytes"));
}

#[test]
fn test_alert_rule_manager_creation() {
    let manager = AlertRuleManager::new();
    assert_eq!(manager.get_rules().len(), 0);
}

#[test]
fn test_add_single_alert_rule() {
    let mut manager = AlertRuleManager::new();

    let rule = AlertRule {
        name: "low_participation".to_string(),
        description: "Alert when participation drops below 80%".to_string(),
        metric: "validator_participation".to_string(),
        condition: "<".to_string(),
        threshold: 80.0,
        duration: 300,
        severity: "Warning".to_string(),
        enabled: true,
    };

    manager.add_rule(rule);
    assert_eq!(manager.get_rules().len(), 1);
    assert_eq!(manager.get_rules()[0].name, "low_participation");
}

#[test]
fn test_add_multiple_alert_rules() {
    let mut manager = AlertRuleManager::new();

    for i in 0..5 {
        let rule = AlertRule {
            name: format!("rule_{}", i),
            description: format!("Test rule {}", i),
            metric: "test_metric".to_string(),
            condition: ">".to_string(),
            threshold: 50.0,
            duration: 300,
            severity: "Warning".to_string(),
            enabled: true,
        };
        manager.add_rule(rule);
    }

    assert_eq!(manager.get_rules().len(), 5);
}

#[test]
fn test_remove_alert_rule() {
    let mut manager = AlertRuleManager::new();

    let rule = AlertRule {
        name: "test_rule".to_string(),
        description: "Test rule".to_string(),
        metric: "test_metric".to_string(),
        condition: ">".to_string(),
        threshold: 50.0,
        duration: 300,
        severity: "Warning".to_string(),
        enabled: true,
    };

    manager.add_rule(rule);
    assert_eq!(manager.get_rules().len(), 1);

    assert!(manager.remove_rule("test_rule"));
    assert_eq!(manager.get_rules().len(), 0);

    assert!(!manager.remove_rule("nonexistent"));
}

#[test]
fn test_enable_disable_rule() {
    let mut manager = AlertRuleManager::new();

    let rule = AlertRule {
        name: "test_rule".to_string(),
        description: "Test rule".to_string(),
        metric: "test_metric".to_string(),
        condition: ">".to_string(),
        threshold: 50.0,
        duration: 300,
        severity: "Warning".to_string(),
        enabled: true,
    };

    manager.add_rule(rule);
    assert!(manager.get_rules()[0].enabled);

    assert!(manager.disable_rule("test_rule"));
    assert!(!manager.get_rules()[0].enabled);

    assert!(manager.enable_rule("test_rule"));
    assert!(manager.get_rules()[0].enabled);
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

    let rule3 = AlertRule {
        name: "rule3".to_string(),
        description: "Rule 3".to_string(),
        metric: "test".to_string(),
        condition: "==".to_string(),
        threshold: 100.0,
        duration: 300,
        severity: "Info".to_string(),
        enabled: true,
    };

    manager.add_rule(rule1);
    manager.add_rule(rule2);
    manager.add_rule(rule3);

    let enabled = manager.get_enabled_rules();
    assert_eq!(enabled.len(), 2);
    assert_eq!(enabled[0].name, "rule1");
    assert_eq!(enabled[1].name, "rule3");
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
    assert!(manager.evaluate_rule(&rule, 50.1));
    assert!(!manager.evaluate_rule(&rule, 50.0));
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
    assert!(manager.evaluate_rule(&rule, 49.9));
    assert!(!manager.evaluate_rule(&rule, 50.0));
    assert!(!manager.evaluate_rule(&rule, 60.0));
}

#[test]
fn test_evaluate_rule_greater_equal() {
    let manager = AlertRuleManager::new();

    let rule = AlertRule {
        name: "test".to_string(),
        description: "Test".to_string(),
        metric: "test".to_string(),
        condition: ">=".to_string(),
        threshold: 50.0,
        duration: 300,
        severity: "Warning".to_string(),
        enabled: true,
    };

    assert!(manager.evaluate_rule(&rule, 60.0));
    assert!(manager.evaluate_rule(&rule, 50.0));
    assert!(!manager.evaluate_rule(&rule, 40.0));
}

#[test]
fn test_evaluate_rule_less_equal() {
    let manager = AlertRuleManager::new();

    let rule = AlertRule {
        name: "test".to_string(),
        description: "Test".to_string(),
        metric: "test".to_string(),
        condition: "<=".to_string(),
        threshold: 50.0,
        duration: 300,
        severity: "Warning".to_string(),
        enabled: true,
    };

    assert!(manager.evaluate_rule(&rule, 40.0));
    assert!(manager.evaluate_rule(&rule, 50.0));
    assert!(!manager.evaluate_rule(&rule, 60.0));
}

#[test]
fn test_evaluate_rule_equal() {
    let manager = AlertRuleManager::new();

    let rule = AlertRule {
        name: "test".to_string(),
        description: "Test".to_string(),
        metric: "test".to_string(),
        condition: "==".to_string(),
        threshold: 50.0,
        duration: 300,
        severity: "Warning".to_string(),
        enabled: true,
    };

    assert!(manager.evaluate_rule(&rule, 50.0));
    assert!(!manager.evaluate_rule(&rule, 50.1));
    assert!(!manager.evaluate_rule(&rule, 49.9));
}

#[test]
fn test_evaluate_rule_not_equal() {
    let manager = AlertRuleManager::new();

    let rule = AlertRule {
        name: "test".to_string(),
        description: "Test".to_string(),
        metric: "test".to_string(),
        condition: "!=".to_string(),
        threshold: 50.0,
        duration: 300,
        severity: "Warning".to_string(),
        enabled: true,
    };

    assert!(manager.evaluate_rule(&rule, 50.1));
    assert!(manager.evaluate_rule(&rule, 49.9));
    assert!(!manager.evaluate_rule(&rule, 50.0));
}

#[test]
fn test_health_checker_creation() {
    let checker = HealthChecker::new();
    let uptime = checker.get_uptime();
    assert!(uptime >= 0);
}

#[test]
fn test_health_check_healthy_status() {
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
    assert_eq!(status.validator_count, 100);
    assert_eq!(status.active_validators, 95);
    assert_eq!(status.critical_validators, 0);
}

#[test]
fn test_health_check_degraded_status() {
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
fn test_health_check_unhealthy_status() {
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
fn test_health_check_high_cpu_unhealthy() {
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
        95.0, // cpu_usage (high)
    );

    assert_eq!(status.status, "Unhealthy");
}

#[test]
fn test_health_check_high_memory_unhealthy() {
    let checker = HealthChecker::new();

    let status = checker.check_health(
        100,  // validator_count
        95,   // active_validators
        0,    // critical_validators
        95.0, // avg_participation
        98.0, // avg_uptime
        50,   // peers_connected
        1024 * 1024 * 1024, // database_size
        17 * 1024 * 1024 * 1024, // memory_usage (high)
        30.0, // cpu_usage
    );

    assert_eq!(status.status, "Unhealthy");
}

#[test]
fn test_health_status_response_structure() {
    let checker = HealthChecker::new();

    let status = checker.check_health(
        100,
        95,
        0,
        95.0,
        98.0,
        50,
        1024 * 1024 * 1024,
        4 * 1024 * 1024 * 1024,
        30.0,
    );

    assert!(!status.status.is_empty());
    assert!(status.timestamp > 0);
    assert_eq!(status.validator_count, 100);
    assert_eq!(status.active_validators, 95);
    assert_eq!(status.critical_validators, 0);
    assert!(status.avg_participation >= 0.0);
    assert!(status.avg_uptime >= 0.0);
}

#[test]
fn test_alert_rule_serialization() {
    let rule = AlertRule {
        name: "test_rule".to_string(),
        description: "Test rule".to_string(),
        metric: "test_metric".to_string(),
        condition: ">".to_string(),
        threshold: 50.0,
        duration: 300,
        severity: "Warning".to_string(),
        enabled: true,
    };

    let json = serde_json::to_string(&rule);
    assert!(json.is_ok());

    let deserialized: Result<AlertRule, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());

    let rule2 = deserialized.unwrap();
    assert_eq!(rule2.name, rule.name);
    assert_eq!(rule2.threshold, rule.threshold);
}

#[test]
fn test_health_check_status_serialization() {
    let checker = HealthChecker::new();

    let status = checker.check_health(
        100,
        95,
        0,
        95.0,
        98.0,
        50,
        1024 * 1024 * 1024,
        4 * 1024 * 1024 * 1024,
        30.0,
    );

    let json = serde_json::to_string(&status);
    assert!(json.is_ok());

    let deserialized: Result<HealthCheckStatus, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());

    let status2 = deserialized.unwrap();
    assert_eq!(status2.status, status.status);
    assert_eq!(status2.validator_count, status.validator_count);
}

#[test]
fn test_multiple_alert_rules_evaluation() {
    let mut manager = AlertRuleManager::new();

    let rules = vec![
        AlertRule {
            name: "high_cpu".to_string(),
            description: "CPU usage high".to_string(),
            metric: "cpu".to_string(),
            condition: ">".to_string(),
            threshold: 80.0,
            duration: 300,
            severity: "Warning".to_string(),
            enabled: true,
        },
        AlertRule {
            name: "low_memory".to_string(),
            description: "Memory low".to_string(),
            metric: "memory".to_string(),
            condition: "<".to_string(),
            threshold: 20.0,
            duration: 300,
            severity: "Critical".to_string(),
            enabled: true,
        },
        AlertRule {
            name: "low_participation".to_string(),
            description: "Participation low".to_string(),
            metric: "participation".to_string(),
            condition: "<".to_string(),
            threshold: 80.0,
            duration: 300,
            severity: "Warning".to_string(),
            enabled: true,
        },
    ];

    for rule in rules {
        manager.add_rule(rule);
    }

    // Test evaluations
    assert!(manager.evaluate_rule(&manager.get_rules()[0], 85.0)); // high_cpu
    assert!(manager.evaluate_rule(&manager.get_rules()[1], 15.0)); // low_memory
    assert!(manager.evaluate_rule(&manager.get_rules()[2], 75.0)); // low_participation
}

#[test]
fn test_alert_rule_manager_default() {
    let manager = AlertRuleManager::default();
    assert_eq!(manager.get_rules().len(), 0);
}

#[test]
fn test_health_checker_default() {
    let checker = HealthChecker::default();
    assert!(checker.get_uptime() >= 0);
}

#[test]
fn test_metrics_exporter_default() {
    let exporter = MetricsExporter::default();
    let metrics = exporter.export_metrics();
    assert!(metrics.is_ok());
}
