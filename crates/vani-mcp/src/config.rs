//! Runtime configuration, resolved from environment with safe zero-cost defaults.

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    /// Solana JSON-RPC endpoint. Defaults to the public Devnet (free).
    pub rpc_url: String,
    /// Optional demo address used when a tool's address field is left empty.
    pub default_address: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let rpc_url = std::env::var("RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        let default_address = std::env::var("VANI_DEFAULT_ADDRESS")
            .ok()
            .filter(|s| !s.trim().is_empty());
        Ok(Self {
            rpc_url,
            default_address,
        })
    }
}