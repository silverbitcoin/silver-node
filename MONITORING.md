# SilverBitcoin Node Monitoring and Metrics

This document describes the monitoring and metrics implementation for the SilverBitcoin node.

## Overview

The node implements comprehensive monitoring capabilities including:
- **Prometheus metrics exporter** for real-time performance tracking
- **Health check endpoints** for operational status monitoring
- **Resource monitoring** with threshold-based alerting

## Requirements Satisfied

### Requirement 15.1: Prometheus Metrics Endpoint
✅ **Implemented**: Prometheus-compatible metrics exposed on dedicated HTTP endpoint (default: `http://localhost:9184/metrics`)

### Requirement 15.2: Comprehensive Metrics Tracking
✅ **Implemented**: Tracks the following metrics:

**Consensus Metrics:**
- `silver_consensus_batches_created_total` - Total batches created
- `silver_consensus_batches_certified_total` - Total batches certified
- `silver_consensus_snapshots_created_total` - Total snapshots created
- `silver_consensus_snapshot_height` - Current snapshot height
- `silver_consensus_latency_milliseconds` - Consensus latency histogram
- `silver_consensus_batch_size_transactions` - Batch size in transactions
- `silver_consensus_batch_size_bytes` - Batch size in bytes
- `silver_consensus_active_validators` - Number of active validators
- `silver_consensus_total_stake` - Total stake weight

**Execution Metrics:**
- `silver_execution_transactions_executed_total` - Total transactions executed
- `silver_execution_transactions_failed_total` - Total transactions failed
- `silver_execution_time_milliseconds` - Transaction execution time
- `silver_execution_fuel_consumed_total` - Total fuel consumed
- `silver_execution_fuel_refunded_total` - Total fuel refunded
- `silver_execution_parallel_efficiency` - Parallel execution efficiency (0-1)
- `silver_execution_active_threads` - Active execution threads

**Storage Metrics:**
- `silver_storage_objects_total` - Total objects stored
- `silver_storage_transactions_total` - Total transactions stored
- `silver_storage_events_total` - Total events stored
- `silver_storage_db_size_bytes` - Database size in bytes
- `silver_storage_cache_hit_rate` - Cache hit rate (0-1)
- `silver_storage_read_ops_total` - Total read operations
- `silver_storage_write_ops_total` - Total write operations
- `silver_storage_read_latency_milliseconds` - Read latency histogram
- `silver_storage_write_latency_milliseconds` - Write latency histogram

**Network Metrics:**
- `silver_network_connected_peers` - Number of connected peers
- `silver_network_messages_sent_total` - Total messages sent
- `silver_network_messages_received_total` - Total messages received
- `silver_network_bytes_sent_total` - Total bytes sent
- `silver_network_bytes_received_total` - Total bytes received
- `silver_network_propagation_latency_milliseconds` - Message propagation latency
- `silver_network_peer_reputation` - Peer reputation scores
- `silver_network_blocked_peers` - Number of blocked peers

**API Metrics:**
- `silver_api_rpc_requests_total` - Total RPC requests
- `silver_api_rpc_requests_by_method_total` - RPC requests by method
- `silver_api_rpc_latency_milliseconds` - RPC request latency
- `silver_api_websocket_connections` - Active WebSocket connections
- `silver_api_active_subscriptions` - Active event subscriptions
- `silver_api_rate_limited_requests_total` - Rate limited requests

**System Metrics:**
- `silver_system_cpu_usage_percent` - CPU usage percentage
- `silver_system_memory_usage_bytes` - Memory usage in bytes
- `silver_system_disk_usage_bytes` - Disk usage in bytes
- `silver_system_disk_available_bytes` - Disk available in bytes
- `silver_system_thread_count` - Number of threads
- `silver_system_file_descriptors` - File descriptors open

### Requirement 15.3: Real-Time Metrics Updates
✅ **Implemented**: Metrics are updated at least once per second (configurable via `update_interval_seconds`)

### Requirement 15.4: Resource Usage Warnings
✅ **Implemented**: Logs warnings when resource utilization exceeds 80% of configured limits:
- CPU usage > 80%
- Memory usage > 80%
- Disk usage > 80%
- File descriptor usage > 80%

Warning cooldown: 5 minutes between repeated warnings for the same resource.

### Requirement 15.5: Health Check Endpoint
✅ **Implemented**: Health check endpoint returns HTTP 200 when node is synchronized and operational

## Health Check Endpoints

The node exposes three health check endpoints (default: `http://localhost:9185`):

### `/health` - Comprehensive Health Status
Returns detailed health information:
```json
{
  "status": "healthy",
  "sync_status": {
    "is_synced": true,
    "current_height": 1000,
    "network_height": 1000,
    "sync_progress": 100.0
  },
  "peer_count": 10,
  "snapshot_height": 1000,
  "uptime_seconds": 3600,
  "version": "0.1.0"
}
```

**Status Values:**
- `healthy` - Node is fully operational (HTTP 200)
- `syncing` - Node is syncing with network (HTTP 200)
- `degraded` - Node is operational but with issues (HTTP 200)
- `unhealthy` - Node is not operational (HTTP 503)

### `/ready` - Readiness Probe
Kubernetes-compatible readiness probe. Returns HTTP 200 when:
- Node is synchronized with network
- At least one peer is connected

### `/live` - Liveness Probe
Kubernetes-compatible liveness probe. Returns HTTP 200 when:
- Node is healthy, syncing, or degraded
- Returns HTTP 503 only when node is unhealthy

## Configuration

Configure monitoring in `node.toml`:

```toml
[metrics]
# Prometheus metrics endpoint
prometheus_address = "0.0.0.0:9184"
enable_metrics = true
# Update metrics at least once per second
update_interval_seconds = 1
```

Health check endpoint is automatically configured on port 9185 (metrics port + 1).

## Usage Examples

### Query Prometheus Metrics
```bash
curl http://localhost:9184/metrics
```

### Check Node Health
```bash
curl http://localhost:9185/health
```

### Check Readiness (for load balancers)
```bash
curl http://localhost:9185/ready
```

### Check Liveness (for container orchestration)
```bash
curl http://localhost:9185/live
```

## Integration with Prometheus

Add to your `prometheus.yml`:

```yaml
scrape_configs:
  - job_name: 'silverbitcoin-node'
    static_configs:
      - targets: ['localhost:9184']
    scrape_interval: 1s
```

## Grafana Dashboard

Recommended panels:
1. **Consensus Performance**
   - Snapshot height (gauge)
   - Consensus latency (graph)
   - Batch creation rate (graph)

2. **Transaction Throughput**
   - Transactions executed per second (graph)
   - Transaction execution time (histogram)
   - Fuel consumption rate (graph)

3. **Network Health**
   - Connected peers (gauge)
   - Message propagation latency (graph)
   - Network bandwidth (graph)

4. **System Resources**
   - CPU usage (gauge)
   - Memory usage (gauge)
   - Disk usage (gauge)
   - File descriptors (gauge)

## Architecture

### Metrics Exporter (`metrics.rs`)
- Initializes all Prometheus metrics
- Exposes HTTP endpoint for scraping
- Updates system metrics periodically
- Thread-safe metric access via Arc<RwLock>

### Health Check Server (`health.rs`)
- Provides three HTTP endpoints: /health, /ready, /live
- Maintains health state shared with node
- Updates health status based on sync and peer count
- Supports Kubernetes probes

### Resource Monitor (`resources.rs`)
- Monitors CPU, memory, disk, and file descriptor usage
- Logs warnings when thresholds exceeded (default: 80%)
- Platform-specific implementations for Linux and macOS
- Configurable warning cooldown (default: 5 minutes)

## Testing

Run tests:
```bash
cargo test --package silver-node --bin silver-node
```

All monitoring components include comprehensive unit tests:
- Metrics initialization and access
- Health state transitions
- Resource threshold checking
- Warning cooldown logic

## Production Considerations

1. **Metrics Retention**: Configure Prometheus retention based on your needs
2. **Alert Rules**: Set up Prometheus alerts for critical metrics
3. **Dashboard**: Create Grafana dashboards for visualization
4. **Log Aggregation**: Integrate with ELK/Loki for centralized logging
5. **Tracing**: Consider adding distributed tracing (Jaeger/Zipkin)

## Future Enhancements

- [ ] Add custom metrics via plugin system
- [ ] Implement metrics aggregation for multi-node deployments
- [ ] Add OpenTelemetry support
- [ ] Implement metrics-based auto-scaling
- [ ] Add anomaly detection for metrics
