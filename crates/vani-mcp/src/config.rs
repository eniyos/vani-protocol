//! Runtime configuration, resolved from environment with safe zero-cost defaults.

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    /// Solana JSON-RPC endpoint. Defaults to the public Devnet (free).
    pub rpc_url: String,
    /// Optional demo address used when a tool's address field is left empty.
    pub default_address: Option<String>,
    /// Sarvam AI API subscription key (voice STT/TTS). Read-only use per
    /// request — never logged or stored anywhere else.
    pub sarvam_api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let rpc_url = std::env::var("RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());
        // Trim env values: accidental trailing whitespace/newlines would make a
        // base58 address invalid and a Sarvam key silently 403.
        let default_address = std::env::var("VANI_DEFAULT_ADDRESS")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let sarvam_api_key = std::env::var("SARVAM_API_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        Ok(Self {
            rpc_url,
            default_address,
            sarvam_api_key,
        })
    }
}