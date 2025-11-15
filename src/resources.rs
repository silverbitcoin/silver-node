//! # Resource Monitoring
//!
//! System resource monitoring with threshold-based alerting.

use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

/// Resource monitoring error types
#[derive(Error, Debug)]
pub enum ResourceError {
    /// Failed to read system information
    #[error("Failed to read system info: {0}")]
    SystemInfoError(String),

    /// Failed to read process information
    #[error("Failed to read process info: {0}")]
    ProcessInfoError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for resource operations
pub type Result<T> = std::result::Result<T, ResourceError>;

/// Resource usage thresholds
#[derive(Debug, Clone)]
pub struct ResourceThresholds {
    /// CPU usage warning threshold (percentage)
    pub cpu_warning_percent: f64,
    
    /// Memory usage warning threshold (percentage)
    pub memory_warning_percent: f64,
    
    /// Disk usage warning threshold (percentage)
    pub disk_warning_percent: f64,
    
    /// File descriptor warning threshold (percentage of max)
    pub fd_warning_percent: f64,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        Self {
            cpu_warning_percent: 80.0,
            memory_warning_percent: 80.0,
            disk_warning_percent: 80.0,
            fd_warning_percent: 80.0,
        }
    }
}

/// CPU usage information
#[derive(Debug, Clone)]
pub struct CpuUsage {
    /// Overall CPU usage percentage (0-100)
    pub usage_percent: f64,
    
    /// Number of CPU cores
    pub core_count: usize,
    
    /// Per-core usage percentages
    pub per_core_usage: Vec<f64>,
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Total memory in bytes
    pub total_bytes: u64,
    
    /// Used memory in bytes
    pub used_bytes: u64,
    
    /// Available memory in bytes
    pub available_bytes: u64,
    
    /// Usage percentage (0-100)
    pub usage_percent: f64,
}

/// Disk usage information
#[derive(Debug, Clone)]
pub struct DiskUsage {
    /// Total disk space in bytes
    pub total_bytes: u64,
    
    /// Used disk space in bytes
    pub used_bytes: u64,
    
    /// Available disk space in bytes
    pub available_bytes: u64,
    
    /// Usage percentage (0-100)
    pub usage_percent: f64,
}

/// Process resource usage
#[derive(Debug, Clone)]
pub struct ProcessUsage {
    /// Process ID
    pub pid: u32,
    
    /// Number of threads
    pub thread_count: usize,
    
    /// Number of open file descriptors
    pub fd_count: usize,
    
    /// Maximum file descriptors
    pub fd_max: usize,
    
    /// File descriptor usage percentage
    pub fd_usage_percent: f64,
}

/// Complete resource usage snapshot
#[derive(Debug, Clone)]
pub struct ResourceSnapshot {
    /// CPU usage
    pub cpu: CpuUsage,
    
    /// Memory usage
    pub memory: MemoryUsage,
    
    /// Disk usage
    pub disk: DiskUsage,
    
    /// Process usage
    pub process: ProcessUsage,
    
    /// Timestamp
    pub timestamp: std::time::SystemTime,
}

/// Resource monitor
pub struct ResourceMonitor {
    /// Resource thresholds
    thresholds: ResourceThresholds,
    
    /// Data directory path for disk monitoring
    data_dir: std::path::PathBuf,
    
    /// Last resource snapshot
    last_snapshot: Arc<RwLock<Option<ResourceSnapshot>>>,
    
    /// Monitoring interval in seconds
    interval_seconds: u64,
    
    /// Warning cooldown (seconds between repeated warnings)
    warning_cooldown_seconds: u64,
    
    /// Last warning times
    last_cpu_warning: Arc<RwLock<Option<std::time::Instant>>>,
    last_memory_warning: Arc<RwLock<Option<std::time::Instant>>>,
    last_disk_warning: Arc<RwLock<Option<std::time::Instant>>>,
    last_fd_warning: Arc<RwLock<Option<std::time::Instant>>>,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(
        thresholds: ResourceThresholds,
        data_dir: std::path::PathBuf,
        interval_seconds: u64,
    ) -> Self {
        Self {
            thresholds,
            data_dir,
            last_snapshot: Arc::new(RwLock::new(None)),
            interval_seconds,
            warning_cooldown_seconds: 300, // 5 minutes
            last_cpu_warning: Arc::new(RwLock::new(None)),
            last_memory_warning: Arc::new(RwLock::new(None)),
            last_disk_warning: Arc::new(RwLock::new(None)),
            last_fd_warning: Arc::new(RwLock::new(None)),
        }
    }

    /// Start resource monitoring loop
    pub async fn start(&self) {
        info!("Starting resource monitor (interval: {}s)", self.interval_seconds);
        info!("Resource thresholds: CPU: {:.1}%, Memory: {:.1}%, Disk: {:.1}%, FD: {:.1}%",
              self.thresholds.cpu_warning_percent,
              self.thresholds.memory_warning_percent,
              self.thresholds.disk_warning_percent,
              self.thresholds.fd_warning_percent);

        let monitor = self.clone_for_task();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(monitor.interval_seconds)
            );

            loop {
                interval.tick().await;

                match monitor.collect_snapshot().await {
                    Ok(snapshot) => {
                        monitor.check_thresholds(&snapshot).await;
                        *monitor.last_snapshot.write().await = Some(snapshot);
                    }
                    Err(e) => {
                        error!("Failed to collect resource snapshot: {}", e);
                    }
                }
            }
        });
    }

    /// Clone for async task
    fn clone_for_task(&self) -> Self {
        Self {
            thresholds: self.thresholds.clone(),
            data_dir: self.data_dir.clone(),
            last_snapshot: self.last_snapshot.clone(),
            interval_seconds: self.interval_seconds,
            warning_cooldown_seconds: self.warning_cooldown_seconds,
            last_cpu_warning: self.last_cpu_warning.clone(),
            last_memory_warning: self.last_memory_warning.clone(),
            last_disk_warning: self.last_disk_warning.clone(),
            last_fd_warning: self.last_fd_warning.clone(),
        }
    }

    /// Collect current resource snapshot
    async fn collect_snapshot(&self) -> Result<ResourceSnapshot> {
        let cpu = Self::get_cpu_usage()?;
        let memory = Self::get_memory_usage()?;
        let disk = Self::get_disk_usage(&self.data_dir)?;
        let process = Self::get_process_usage()?;

        Ok(ResourceSnapshot {
            cpu,
            memory,
            disk,
            process,
            timestamp: std::time::SystemTime::now(),
        })
    }

    /// Check resource thresholds and log warnings
    async fn check_thresholds(&self, snapshot: &ResourceSnapshot) {
        // Check CPU usage
        if snapshot.cpu.usage_percent >= self.thresholds.cpu_warning_percent {
            if self.should_warn(&self.last_cpu_warning).await {
                warn!(
                    "CPU usage high: {:.1}% (threshold: {:.1}%)",
                    snapshot.cpu.usage_percent,
                    self.thresholds.cpu_warning_percent
                );
                *self.last_cpu_warning.write().await = Some(std::time::Instant::now());
            }
        }

        // Check memory usage
        if snapshot.memory.usage_percent >= self.thresholds.memory_warning_percent {
            if self.should_warn(&self.last_memory_warning).await {
                warn!(
                    "Memory usage high: {:.1}% ({} / {} MB) (threshold: {:.1}%)",
                    snapshot.memory.usage_percent,
                    snapshot.memory.used_bytes / 1024 / 1024,
                    snapshot.memory.total_bytes / 1024 / 1024,
                    self.thresholds.memory_warning_percent
                );
                *self.last_memory_warning.write().await = Some(std::time::Instant::now());
            }
        }

        // Check disk usage
        if snapshot.disk.usage_percent >= self.thresholds.disk_warning_percent {
            if self.should_warn(&self.last_disk_warning).await {
                warn!(
                    "Disk usage high: {:.1}% ({} / {} GB) (threshold: {:.1}%)",
                    snapshot.disk.usage_percent,
                    snapshot.disk.used_bytes / 1024 / 1024 / 1024,
                    snapshot.disk.total_bytes / 1024 / 1024 / 1024,
                    self.thresholds.disk_warning_percent
                );
                *self.last_disk_warning.write().await = Some(std::time::Instant::now());
            }
        }

        // Check file descriptor usage
        if snapshot.process.fd_usage_percent >= self.thresholds.fd_warning_percent {
            if self.should_warn(&self.last_fd_warning).await {
                warn!(
                    "File descriptor usage high: {:.1}% ({} / {}) (threshold: {:.1}%)",
                    snapshot.process.fd_usage_percent,
                    snapshot.process.fd_count,
                    snapshot.process.fd_max,
                    self.thresholds.fd_warning_percent
                );
                *self.last_fd_warning.write().await = Some(std::time::Instant::now());
            }
        }
    }

    /// Check if enough time has passed since last warning
    async fn should_warn(&self, last_warning: &Arc<RwLock<Option<std::time::Instant>>>) -> bool {
        let guard = last_warning.read().await;
        match *guard {
            None => true,
            Some(last) => {
                last.elapsed().as_secs() >= self.warning_cooldown_seconds
            }
        }
    }

    /// Get last resource snapshot
    pub async fn last_snapshot(&self) -> Option<ResourceSnapshot> {
        self.last_snapshot.read().await.clone()
    }

    /// Get CPU usage
    fn get_cpu_usage() -> Result<CpuUsage> {
        // On Linux, read /proc/stat
        #[cfg(target_os = "linux")]
        {
            Self::get_cpu_usage_linux()
        }

        // On macOS, use sysctl or similar
        #[cfg(target_os = "macos")]
        {
            Self::get_cpu_usage_macos()
        }

        // Fallback for other platforms
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Ok(CpuUsage {
                usage_percent: 0.0,
                core_count: num_cpus::get(),
                per_core_usage: vec![],
            })
        }
    }

    #[cfg(target_os = "linux")]
    fn get_cpu_usage_linux() -> Result<CpuUsage> {
        // Read /proc/stat for CPU usage
        // This is a simplified implementation
        // In production, you'd want to track deltas over time
        let core_count = num_cpus::get();
        
        Ok(CpuUsage {
            usage_percent: 0.0, // Placeholder - would need delta calculation
            core_count,
            per_core_usage: vec![0.0; core_count],
        })
    }

    #[cfg(target_os = "macos")]
    fn get_cpu_usage_macos() -> Result<CpuUsage> {
        // Use host_processor_info or similar
        let core_count = num_cpus::get();
        
        Ok(CpuUsage {
            usage_percent: 0.0, // Placeholder
            core_count,
            per_core_usage: vec![0.0; core_count],
        })
    }

    /// Get memory usage
    fn get_memory_usage() -> Result<MemoryUsage> {
        #[cfg(target_os = "linux")]
        {
            Self::get_memory_usage_linux()
        }

        #[cfg(target_os = "macos")]
        {
            Self::get_memory_usage_macos()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Ok(MemoryUsage {
                total_bytes: 0,
                used_bytes: 0,
                available_bytes: 0,
                usage_percent: 0.0,
            })
        }
    }

    #[cfg(target_os = "linux")]
    fn get_memory_usage_linux() -> Result<MemoryUsage> {
        // Read /proc/meminfo
        let meminfo = std::fs::read_to_string("/proc/meminfo")?;
        
        let mut total = 0u64;
        let mut available = 0u64;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0) * 1024; // Convert KB to bytes
            } else if line.starts_with("MemAvailable:") {
                available = line.split_whitespace()
                    .nth(1)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0) * 1024; // Convert KB to bytes
            }
        }
        
        let used = total.saturating_sub(available);
        let usage_percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Ok(MemoryUsage {
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            usage_percent,
        })
    }

    #[cfg(target_os = "macos")]
    fn get_memory_usage_macos() -> Result<MemoryUsage> {
        // Use vm_stat or sysctl
        // Placeholder implementation
        Ok(MemoryUsage {
            total_bytes: 0,
            used_bytes: 0,
            available_bytes: 0,
            usage_percent: 0.0,
        })
    }

    /// Get disk usage
    fn get_disk_usage(path: &Path) -> Result<DiskUsage> {
        #[cfg(unix)]
        {
            Self::get_disk_usage_unix(path)
        }

        #[cfg(not(unix))]
        {
            Ok(DiskUsage {
                total_bytes: 0,
                used_bytes: 0,
                available_bytes: 0,
                usage_percent: 0.0,
            })
        }
    }

    #[cfg(unix)]
    fn get_disk_usage_unix(path: &Path) -> Result<DiskUsage> {
        use std::os::unix::fs::MetadataExt;
        
        // Use statvfs
        let metadata = std::fs::metadata(path)?;
        
        // This is a simplified version
        // In production, use nix crate's statvfs
        let total_bytes = 0u64; // Would get from statvfs
        let available_bytes = 0u64; // Would get from statvfs
        let used_bytes = total_bytes.saturating_sub(available_bytes);
        let usage_percent = if total_bytes > 0 {
            (used_bytes as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };

        Ok(DiskUsage {
            total_bytes,
            used_bytes,
            available_bytes,
            usage_percent,
        })
    }

    /// Get process resource usage
    fn get_process_usage() -> Result<ProcessUsage> {
        let pid = std::process::id();
        
        #[cfg(target_os = "linux")]
        {
            Self::get_process_usage_linux(pid)
        }

        #[cfg(target_os = "macos")]
        {
            Self::get_process_usage_macos(pid)
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Ok(ProcessUsage {
                pid,
                thread_count: 0,
                fd_count: 0,
                fd_max: 1024,
                fd_usage_percent: 0.0,
            })
        }
    }

    #[cfg(target_os = "linux")]
    fn get_process_usage_linux(pid: u32) -> Result<ProcessUsage> {
        // Count threads from /proc/self/task
        let thread_count = std::fs::read_dir(format!("/proc/{}/task", pid))
            .map(|entries| entries.count())
            .unwrap_or(0);

        // Count file descriptors from /proc/self/fd
        let fd_count = std::fs::read_dir(format!("/proc/{}/fd", pid))
            .map(|entries| entries.count())
            .unwrap_or(0);

        // Get max file descriptors from /proc/self/limits
        let fd_max = Self::get_fd_limit_linux(pid).unwrap_or(1024);

        let fd_usage_percent = if fd_max > 0 {
            (fd_count as f64 / fd_max as f64) * 100.0
        } else {
            0.0
        };

        Ok(ProcessUsage {
            pid,
            thread_count,
            fd_count,
            fd_max,
            fd_usage_percent,
        })
    }

    #[cfg(target_os = "linux")]
    fn get_fd_limit_linux(pid: u32) -> Result<usize> {
        let limits = std::fs::read_to_string(format!("/proc/{}/limits", pid))?;
        
        for line in limits.lines() {
            if line.contains("Max open files") {
                if let Some(soft_limit) = line.split_whitespace().nth(3) {
                    if let Ok(limit) = soft_limit.parse::<usize>() {
                        return Ok(limit);
                    }
                }
            }
        }
        
        Ok(1024) // Default fallback
    }

    #[cfg(target_os = "macos")]
    fn get_process_usage_macos(pid: u32) -> Result<ProcessUsage> {
        // Use proc_pidinfo or similar
        // Placeholder implementation
        Ok(ProcessUsage {
            pid,
            thread_count: 0,
            fd_count: 0,
            fd_max: 1024,
            fd_usage_percent: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_thresholds_default() {
        let thresholds = ResourceThresholds::default();
        assert_eq!(thresholds.cpu_warning_percent, 80.0);
        assert_eq!(thresholds.memory_warning_percent, 80.0);
        assert_eq!(thresholds.disk_warning_percent, 80.0);
        assert_eq!(thresholds.fd_warning_percent, 80.0);
    }

    #[tokio::test]
    async fn test_resource_monitor_creation() {
        let thresholds = ResourceThresholds::default();
        let monitor = ResourceMonitor::new(
            thresholds,
            std::path::PathBuf::from("/tmp"),
            5,
        );

        assert_eq!(monitor.interval_seconds, 5);
        assert!(monitor.last_snapshot.read().await.is_none());
    }

    #[tokio::test]
    async fn test_should_warn_cooldown() {
        let thresholds = ResourceThresholds::default();
        let monitor = ResourceMonitor::new(
            thresholds,
            std::path::PathBuf::from("/tmp"),
            5,
        );

        // First warning should be allowed
        assert!(monitor.should_warn(&monitor.last_cpu_warning).await);

        // Set last warning time
        *monitor.last_cpu_warning.write().await = Some(std::time::Instant::now());

        // Second warning should be blocked (within cooldown)
        assert!(!monitor.should_warn(&monitor.last_cpu_warning).await);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_memory_usage_linux() {
        let result = ResourceMonitor::get_memory_usage_linux();
        assert!(result.is_ok());
        
        let usage = result.unwrap();
        assert!(usage.total_bytes > 0);
        assert!(usage.usage_percent >= 0.0 && usage.usage_percent <= 100.0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_process_usage_linux() {
        let pid = std::process::id();
        let result = ResourceMonitor::get_process_usage_linux(pid);
        assert!(result.is_ok());
        
        let usage = result.unwrap();
        assert_eq!(usage.pid, pid);
        assert!(usage.thread_count > 0);
        assert!(usage.fd_max > 0);
    }
}
