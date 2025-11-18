//! # Genesis Configuration
//!
//! Genesis state initialization for SilverBitcoin blockchain.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Genesis error types
#[derive(Error, Debug)]
pub enum GenesisError {
    /// Failed to load genesis file
    #[error("Failed to load genesis file: {0}")]
    LoadError(#[from] std::io::Error),

    /// Failed to parse genesis file
    #[error("Failed to parse genesis: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Invalid genesis configuration
    #[error("Invalid genesis configuration: {0}")]
    ValidationError(String),
}

/// Result type for genesis operations
pub type Result<T> = std::result::Result<T, GenesisError>;

/// Genesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Chain identifier
    pub chain_id: String,

    /// Genesis timestamp
    pub genesis_time: String,

    /// Protocol version
    pub protocol_version: ProtocolVersion,

    /// Initial validators
    pub validators: Vec<GenesisValidator>,

    /// Initial token supply
    pub initial_supply: u64,

    /// Initial account balances
    pub initial_accounts: Vec<GenesisAccount>,

    /// Consensus configuration
    pub consensus_config: GenesisConsensusConfig,

    /// Fuel configuration
    pub fuel_config: GenesisFuelConfig,

    /// Network configuration
    pub network_config: GenesisNetworkConfig,
}

/// Protocol version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Major version
    pub major: u64,

    /// Minor version
    pub minor: u64,
}

/// Genesis validator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisValidator {
    /// Validator address
    pub address: String,

    /// Protocol public key
    pub protocol_pubkey: String,

    /// Network public key
    pub network_pubkey: String,

    /// Worker public key
    pub worker_pubkey: String,

    /// Stake amount
    pub stake_amount: u64,

    /// Network address
    pub network_address: String,

    /// P2P address
    pub p2p_address: String,
}

/// Genesis account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAccount {
    /// Account address
    pub address: String,

    /// Initial balance
    pub balance: u64,
}

/// Genesis consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConsensusConfig {
    /// Snapshot interval in milliseconds
    pub snapshot_interval_ms: u64,

    /// Maximum transactions per batch
    pub max_batch_transactions: usize,

    /// Maximum batch size in bytes
    pub max_batch_size_bytes: usize,

    /// Byzantine fault tolerance threshold (0.33 = 1/3)
    pub byzantine_fault_tolerance: f64,
}

/// Genesis fuel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisFuelConfig {
    /// Minimum fuel price
    pub min_fuel_price: u64,

    /// Maximum fuel per transaction
    pub max_fuel_per_transaction: u64,
}

/// Genesis network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisNetworkConfig {
    /// Maximum peers
    pub max_peers: usize,

    /// Message rate limit
    pub message_rate_limit: u32,
}

impl GenesisConfig {
    /// Load genesis configuration from JSON file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: GenesisConfig = serde_json::from_str(&contents)?;
        
        // Validate genesis configuration
        config.validate()?;
        
        Ok(config)
    }

    /// Validate genesis configuration
    fn validate(&self) -> Result<()> {
        // Validate chain ID
        if self.chain_id.is_empty() {
            return Err(GenesisError::ValidationError(
                "Chain ID cannot be empty".to_string()
            ));
        }

        // Validate validators
        if self.validators.is_empty() {
            return Err(GenesisError::ValidationError(
                "At least one validator is required".to_string()
            ));
        }

        // Validate validator stakes
        for validator in &self.validators {
            if validator.stake_amount < 1_000_000 {
                return Err(GenesisError::ValidationError(
                    format!("Validator {} stake must be at least 1,000,000 SBTC", validator.address)
                ));
            }
        }

        // Validate initial supply
        if self.initial_supply == 0 {
            return Err(GenesisError::ValidationError(
                "Initial supply must be greater than 0".to_string()
            ));
        }

        // Validate account balances sum doesn't exceed supply
        let total_allocated: u64 = self.initial_accounts.iter()
            .map(|a| a.balance)
            .sum();
        
        let total_staked: u64 = self.validators.iter()
            .map(|v| v.stake_amount)
            .sum();

        if total_allocated + total_staked > self.initial_supply {
            return Err(GenesisError::ValidationError(
                "Total allocated + staked exceeds initial supply".to_string()
            ));
        }

        // Validate consensus config
        if self.consensus_config.snapshot_interval_ms == 0 {
            return Err(GenesisError::ValidationError(
                "Snapshot interval must be greater than 0".to_string()
            ));
        }

        if self.consensus_config.byzantine_fault_tolerance <= 0.0 
            || self.consensus_config.byzantine_fault_tolerance >= 0.5 {
            return Err(GenesisError::ValidationError(
                "Byzantine fault tolerance must be between 0 and 0.5".to_string()
            ));
        }

        // Validate fuel config
        if self.fuel_config.min_fuel_price == 0 {
            return Err(GenesisError::ValidationError(
                "Minimum fuel price must be greater than 0".to_string()
            ));
        }

        if self.fuel_config.max_fuel_per_transaction == 0 {
            return Err(GenesisError::ValidationError(
                "Maximum fuel per transaction must be greater than 0".to_string()
            ));
        }

        Ok(())
    }

    /// Get total validator stake
    pub fn total_stake(&self) -> u64 {
        self.validators.iter().map(|v| v.stake_amount).sum()
    }

    /// Get validator count
    pub fn validator_count(&self) -> usize {
        self.validators.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_genesis() -> GenesisConfig {
        GenesisConfig {
            chain_id: "test-chain".to_string(),
            genesis_time: "2024-01-01T00:00:00Z".to_string(),
            protocol_version: ProtocolVersion { major: 1, minor: 0 },
            validators: vec![
                GenesisValidator {
                    address: "0x01".to_string(),
                    protocol_pubkey: "0xabc".to_string(),
                    network_pubkey: "0xdef".to_string(),
                    worker_pubkey: "0x123".to_string(),
                    stake_amount: 10_000_000,
                    network_address: "/ip4/127.0.0.1/tcp/9000".to_string(),
                    p2p_address: "/ip4/127.0.0.1/tcp/9001".to_string(),
                }
            ],
            initial_supply: 21_000_000_000_000,
            initial_accounts: vec![
                GenesisAccount {
                    address: "0x02".to_string(),
                    balance: 1_000_000_000,
                }
            ],
            consensus_config: GenesisConsensusConfig {
                snapshot_interval_ms: 480,
                max_batch_transactions: 500,
                max_batch_size_bytes: 524288,
                byzantine_fault_tolerance: 0.33,
            },
            fuel_config: GenesisFuelConfig {
                min_fuel_price: 1000,
                max_fuel_per_transaction: 50_000_000,
            },
            network_config: GenesisNetworkConfig {
                max_peers: 50,
                message_rate_limit: 10000,
            },
        }
    }

    #[test]
    fn test_valid_genesis() {
        let genesis = create_test_genesis();
        assert!(genesis.validate().is_ok());
    }

    #[test]
    fn test_empty_chain_id() {
        let mut genesis = create_test_genesis();
        genesis.chain_id = String::new();
        assert!(genesis.validate().is_err());
    }

    #[test]
    fn test_no_validators() {
        let mut genesis = create_test_genesis();
        genesis.validators.clear();
        assert!(genesis.validate().is_err());
    }

    #[test]
    fn test_insufficient_validator_stake() {
        let mut genesis = create_test_genesis();
        genesis.validators[0].stake_amount = 100_000; // Less than minimum
        assert!(genesis.validate().is_err());
    }

    #[test]
    fn test_total_stake() {
        let genesis = create_test_genesis();
        assert_eq!(genesis.total_stake(), 10_000_000);
    }
}
