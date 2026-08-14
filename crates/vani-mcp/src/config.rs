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
    /// Turnkey organization ID — the TEE custody account toward which signing
    /// requests are submitted. None means the execution tools are disabled.
    pub turnkey_org: Option<String>,
    /// Compressed P-256 public key hex of the Turnkey API key pair (auth).
    pub turnkey_api_public: Option<String>,
    /// Raw EC private key hex of the Turnkey API key pair (auth stamping).
    /// Used only to sign request bodies in memory; never stored or logged.
    pub turnkey_api_private: Option<String>,
    /// Derived Solana address of the Turnkey wallet used as signer + fee payer.
    pub turnkey_sol_wallet: Option<String>,
    /// Network for `vani_execute` (CAIP-2). Defaults to mainnet-beta.
    pub execute_network: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();
        let rpc_url = std::env::var("RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());
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
        // Turnkey execution (Week 3). All optional: absent ⇒ the execution tools
        // return a clear "not configured" error, mirroring the voice tools.
        let turnkey_org = std::env::var("TURNKEY_ORGANIZATION_ID")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let turnkey_api_public = std::env::var("TURNKEY_API_PUBLIC_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let turnkey_api_private = std::env::var("TURNKEY_API_PRIVATE_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let turnkey_sol_wallet = std::env::var("TURNKEY_SOLANA_WALLET_ADDRESS")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let execute_network = std::env::var("VANI_EXECUTE_NETWORK")
            .unwrap_or_else(|_| "solana:mainnet-beta".to_string());
        Ok(Self {
            rpc_url,
            default_address,
            sarvam_api_key,
            turnkey_org,
            turnkey_api_public,
            turnkey_api_private,
            turnkey_sol_wallet,
            execute_network,
        })
    }
}