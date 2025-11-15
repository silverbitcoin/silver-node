//! # Memory Management for Consumer Hardware
//!
//! This module provides memory monitoring and management for running
//! SilverBitcoin nodes on consumer-grade hardware (16GB RAM).
//!
//! Features:
//! - Real-time memory usage monitoring
//! - Automatic cache eviction when memory is low
//! - Emergency cleanup procedures
//! - Memory usage warnings and alerts
//! - Adaptive performance tuning based on available memory

use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt};
use tracing::{debug, error, info, warn};

/// Memory management configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Maximum memory usage in bytes (default: 8GB for 16GB systems)
    pub max_memory_usage: u64,
    
    /// Warning threshold (0.0 to 1.0)
    pub warning_threshold: f64,
    
    /// Critical threshold (0.0 to 1.0)
    pub critical_threshold: f64,
    
    /// Memory check interval
    pub check_interval: Duration,
    
    /// Enable memory monitoring
    pub enabled: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_memory_usage: 8 * 1024 * 1024 * 1024, // 8GB
            warning_threshold: 0.85,
            critical_threshold: 0.95,
            check_interval: Duration::from_secs(30),
            enabled: true,
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Total system memory in bytes
    pub total_memory: u64,
    
    /// Used memory in bytes
    pub used_memory: u64,
    
    /// Available memory in bytes
    pub available_memory: u64,
    
    /// Process memory usage in bytes
    pub process_memory: u64,
    
    /// Memory usage percentage (0.0 to 1.0)
    pub usage_percentage: f64,
    
    /// Last update timestamp
    pub last_update: Instant,
}

/// Memory pressure level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryPressure {
    /// Normal operation - plenty of memory available
    Normal,
    
    /// Warning level - approaching memory limits
    Warning,
    
    /// Critical level - very low memory, emergency cleanup needed
    Critical,
}

/// Memory manager for consumer hardware optimization
pub struct MemoryManager {
    /// Configuration
    config: MemoryConfig,
    
    /// System information
    system: Arc<RwLock<System>>,
    
    /// Current memory statistics
    stats: Arc<RwLock<MemoryStats>>,
    
    /// Current memory pressure level
    pressure: Arc<RwLock<MemoryPressure>>,
    
    /// Cleanup callbacks
    cleanup_callbacks: Arc<RwLock<Vec<Box<dyn Fn() + Send + Sync>>>>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(config: MemoryConfig) -> Self {
        info!(
            "Initializing MemoryManager with max_memory={}GB, warning={}%, critical={}%",
            config.max_memory_usage / (1024 * 1024 * 1024),
            (config.warning_threshold * 100.0) as u32,
            (config.critical_threshold * 100.0) as u32
        );
        
        let mut system = System::new_all();
        system.refresh_all();
        
        let total_memory = system.total_memory() * 1024; // Convert KB to bytes
        let used_memory = system.used_memory() * 1024;
        let available_memory = system.available_memory() * 1024;
        
        info!(
            "System memory: total={}GB, used={}GB, available={}GB",
            total_memory / (1024 * 1024 * 1024),
            used_memory / (1024 * 1024 * 1024),
            available_memory / (1024 * 1024 * 1024)
        );
        
        let stats = MemoryStats {
            total_memory,
            used_memory,
            available_memory,
            process_memory: 0,
            usage_percentage: 0.0,
            last_update: Instant::now(),
        };
        
        Self {
            config,
            system: Arc::new(RwLock::new(system)),
            stats: Arc::new(RwLock::new(stats)),
            pressure: Arc::new(RwLock::new(MemoryPressure::Normal)),
            cleanup_callbacks: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Register a cleanup callback to be called when memory pressure is high
    pub fn register_cleanup_callback<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let mut callbacks = self.cleanup_callbacks.write();
        callbacks.push(Box::new(callback));
        debug!("Registered memory cleanup callback (total: {})", callbacks.len());
    }
    
    /// Update memory statistics
    pub fn update_stats(&self) {
        if !self.config.enabled {
            return;
        }
        
        let mut system = self.system.write();
        system.refresh_memory();
        
        let total_memory = system.total_memory() * 1024;
        let used_memory = system.used_memory() * 1024;
        let available_memory = system.available_memory() * 1024;
        
        // Get process memory usage
        let process_memory = self.get_process_memory(&system);
        
        let usage_percentage = process_memory as f64 / self.config.max_memory_usage as f64;
        
        let mut stats = self.stats.write();
        *stats = MemoryStats {
            total_memory,
            used_memory,
            available_memory,
            process_memory,
            usage_percentage,
            last_update: Instant::now(),
        };
        
        // Update pressure level
        self.update_pressure_level(usage_percentage);
        
        debug!(
            "Memory stats: process={}MB, usage={:.1}%, available={}MB",
            process_memory / (1024 * 1024),
            usage_percentage * 100.0,
            available_memory / (1024 * 1024)
        );
    }
    
    /// Get current process memory usage
    fn get_process_memory(&self, system: &System) -> u64 {
        use sysinfo::{Pid, ProcessExt};
        
        let pid = Pid::from(std::process::id() as usize);
        if let Some(process) = system.process(pid) {
            process.memory() * 1024 // Convert KB to bytes
        } else {
            0
        }
    }
    
    /// Update memory pressure level and trigger actions if needed
    fn update_pressure_level(&self, usage_percentage: f64) {
        let new_pressure = if usage_percentage >= self.config.critical_threshold {
            MemoryPressure::Critical
        } else if usage_percentage >= self.config.warning_threshold {
            MemoryPressure::Warning
        } else {
            MemoryPressure::Normal
        };
        
        let mut pressure = self.pressure.write();
        let old_pressure = *pressure;
        
        if new_pressure != old_pressure {
            match new_pressure {
                MemoryPressure::Normal => {
                    info!("Memory pressure: NORMAL ({}%)", (usage_percentage * 100.0) as u32);
                }
                MemoryPressure::Warning => {
                    warn!(
                        "Memory pressure: WARNING ({}%) - approaching memory limits",
                        (usage_percentage * 100.0) as u32
                    );
                }
                MemoryPressure::Critical => {
                    error!(
                        "Memory pressure: CRITICAL ({}%) - triggering emergency cleanup",
                        (usage_percentage * 100.0) as u32
                    );
                    self.trigger_cleanup();
                }
            }
            
            *pressure = new_pressure;
        }
    }
    
    /// Trigger cleanup callbacks
    fn trigger_cleanup(&self) {
        info!("Triggering memory cleanup callbacks");
        
        let callbacks = self.cleanup_callbacks.read();
        for callback in callbacks.iter() {
            callback();
        }
        
        info!("Memory cleanup completed");
    }
    
    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        self.stats.read().clone()
    }
    
    /// Get current memory pressure level
    pub fn get_pressure(&self) -> MemoryPressure {
        *self.pressure.read()
    }
    
    /// Check if memory usage is within limits
    pub fn is_memory_available(&self, required_bytes: u64) -> bool {
        let stats = self.stats.read();
        let available = self.config.max_memory_usage.saturating_sub(stats.process_memory);
        available >= required_bytes
    }
    
    /// Get available memory in bytes
    pub fn get_available_memory(&self) -> u64 {
        let stats = self.stats.read();
        self.config.max_memory_usage.saturating_sub(stats.process_memory)
    }
    
    /// Start memory monitoring loop
    pub fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
        let manager = self.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(manager.config.check_interval);
            
            loop {
                interval.tick().await;
                manager.update_stats();
            }
        })
    }
    
    /// Force garbage collection and cleanup
    pub fn force_cleanup(&self) {
        warn!("Forcing memory cleanup");
        self.trigger_cleanup();
        
        // Suggest garbage collection (Rust doesn't have explicit GC, but this helps)
        // In practice, this triggers cleanup callbacks which should free memory
    }
}

impl Clone for MemoryManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            system: Arc::clone(&self.system),
            stats: Arc::clone(&self.stats),
            pressure: Arc::clone(&self.pressure),
            cleanup_callbacks: Arc::clone(&self.cleanup_callbacks),
        }
    }
}

/// Memory-aware cache that respects memory limits
pub struct MemoryAwareCache<K, V> {
    /// Underlying cache
    cache: Arc<RwLock<lru::LruCache<K, V>>>,
    
    /// Memory manager
    memory_manager: Arc<MemoryManager>,
    
    /// Estimated size per entry in bytes
    entry_size: usize,
}

impl<K: std::hash::Hash + Eq + Clone, V: Clone> MemoryAwareCache<K, V> {
    /// Create a new memory-aware cache
    pub fn new(
        max_entries: usize,
        entry_size: usize,
        memory_manager: Arc<MemoryManager>,
    ) -> Self {
        let cache = lru::LruCache::new(max_entries);
        
        let cache_instance = Self {
            cache: Arc::new(RwLock::new(cache)),
            memory_manager: Arc::clone(&memory_manager),
            entry_size,
        };
        
        // Register cleanup callback
        let cache_clone = Arc::clone(&cache_instance.cache);
        memory_manager.register_cleanup_callback(move || {
            let mut cache = cache_clone.write();
            let old_len = cache.len();
            
            // Evict 50% of entries when memory pressure is critical
            let target_len = old_len / 2;
            while cache.len() > target_len {
                cache.pop_lru();
            }
            
            info!(
                "Memory cleanup: evicted {} cache entries ({}MB freed)",
                old_len - cache.len(),
                ((old_len - cache.len()) * entry_size) / (1024 * 1024)
            );
        });
        
        cache_instance
    }
    
    /// Get an entry from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();
        cache.get(key).cloned()
    }
    
    /// Put an entry into the cache (respecting memory limits)
    pub fn put(&self, key: K, value: V) -> bool {
        // Check if we have enough memory
        if !self.memory_manager.is_memory_available(self.entry_size as u64) {
            debug!("Insufficient memory for cache entry, skipping");
            return false;
        }
        
        let mut cache = self.cache.write();
        cache.put(key, value);
        true
    }
    
    /// Clear the cache
    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.clear();
    }
    
    /// Get cache size
    pub fn len(&self) -> usize {
        let cache = self.cache.read();
        cache.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert_eq!(config.max_memory_usage, 8 * 1024 * 1024 * 1024);
        assert_eq!(config.warning_threshold, 0.85);
        assert_eq!(config.critical_threshold, 0.95);
    }
    
    #[test]
    fn test_memory_manager_creation() {
        let config = MemoryConfig::default();
        let manager = MemoryManager::new(config);
        
        let stats = manager.get_stats();
        assert!(stats.total_memory > 0);
        assert_eq!(manager.get_pressure(), MemoryPressure::Normal);
    }
    
    #[test]
    fn test_memory_pressure_levels() {
        let config = MemoryConfig {
            max_memory_usage: 1000,
            warning_threshold: 0.8,
            critical_threshold: 0.9,
            check_interval: Duration::from_secs(30),
            enabled: true,
        };
        
        let manager = MemoryManager::new(config);
        
        // Test normal pressure
        manager.update_pressure_level(0.5);
        assert_eq!(manager.get_pressure(), MemoryPressure::Normal);
        
        // Test warning pressure
        manager.update_pressure_level(0.85);
        assert_eq!(manager.get_pressure(), MemoryPressure::Warning);
        
        // Test critical pressure
        manager.update_pressure_level(0.95);
        assert_eq!(manager.get_pressure(), MemoryPressure::Critical);
    }
}
