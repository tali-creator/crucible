//! Mock environment for Soroban contract testing.
//!
//! Provides `MockEnv` - a wrapper around `soroban_sdk::Env` with convenient
//! helpers for testing, and `MockEnvBuilder` for fluent environment construction.

use soroban_sdk::{testutils::Ledger, Address, Env};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A wrapper around the Soroban test environment with additional helpers.
pub struct MockEnv {
    inner: Env,
    accounts: Rc<RefCell<HashMap<String, Address>>>,
}

impl MockEnv {
    /// Returns the underlying `soroban_sdk::Env`.
    pub fn inner(&self) -> &Env {
        &self.inner
    }

    /// Creates a new `MockEnvBuilder` for fluent environment construction.
    pub fn builder() -> MockEnvBuilder {
        MockEnvBuilder::new()
    }

    /// Get an account address by name.
    pub fn account(&self, name: &str) -> Address {
        self.accounts
            .borrow()
            .get(name)
            .cloned()
            .unwrap_or_else(|| panic!("Account '{}' not found", name))
    }

    /// Register an account with a name.
    pub fn register_account(&self, name: &str, address: Address) {
        self.accounts.borrow_mut().insert(name.to_string(), address);
    }
}

impl Default for MockEnv {
    fn default() -> Self {
        Self {
            inner: Env::default(),
            accounts: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

/// Builder for constructing a `MockEnv` with custom configuration.
pub struct MockEnvBuilder {
    env: MockEnv,
}

impl MockEnvBuilder {
    fn new() -> Self {
        Self {
            env: MockEnv::default(),
        }
    }

    /// Set the ledger sequence number.
    pub fn at_sequence(mut self, sequence: u32) -> Self {
        let info = self.env.inner.ledger().get();
        self.env
            .inner
            .ledger()
            .set(soroban_sdk::testutils::LedgerInfo {
                sequence_number: sequence,
                timestamp: info.timestamp,
                protocol_version: info.protocol_version,
                base_reserve: info.base_reserve,
                network_id: info.network_id,
                min_temp_entry_ttl: info.min_temp_entry_ttl,
                min_persistent_entry_ttl: info.min_persistent_entry_ttl,
                max_entry_ttl: info.max_entry_ttl,
            });
        self
    }

    /// Set the ledger timestamp.
    pub fn at_timestamp(mut self, timestamp: u64) -> Self {
        let info = self.env.inner.ledger().get();
        self.env
            .inner
            .ledger()
            .set(soroban_sdk::testutils::LedgerInfo {
                sequence_number: info.sequence_number,
                timestamp: timestamp,
                protocol_version: info.protocol_version,
                base_reserve: info.base_reserve,
                network_id: info.network_id,
                min_temp_entry_ttl: info.min_temp_entry_ttl,
                min_persistent_entry_ttl: info.min_persistent_entry_ttl,
                max_entry_ttl: info.max_entry_ttl,
            });
        self
    }

    /// Set the protocol version.
    pub fn with_protocol_version(mut self, version: u32) -> Self {
        let info = self.env.inner.ledger().get();
        self.env
            .inner
            .ledger()
            .set(soroban_sdk::testutils::LedgerInfo {
                sequence_number: info.sequence_number,
                timestamp: info.timestamp,
                protocol_version: version,
                base_reserve: info.base_reserve,
                network_id: info.network_id,
                min_temp_entry_ttl: info.min_temp_entry_ttl,
                min_persistent_entry_ttl: info.min_persistent_entry_ttl,
                max_entry_ttl: info.max_entry_ttl,
            });
        self
    }

    /// Add a named account with a mock auth contract.
    pub fn with_account(self, name: &str, _balance: i128) -> Self {
        // Create a mock auth contract for the account
        let address = self
            .env
            .inner
            .register_contract::<soroban_sdk::testutils::MockAuthContract>(
                None,
                soroban_sdk::testutils::MockAuthContract,
            );
        self.env.register_account(name, address);
        self
    }

    /// Build the `MockEnv`.
    pub fn build(self) -> MockEnv {
        self.env
    }
}
