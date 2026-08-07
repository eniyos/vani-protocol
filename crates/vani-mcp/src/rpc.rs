//! Minimal Solana JSON-RPC client over any configured endpoint (reqwest).

use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::{json, Value};

/// A thin client for the Solana JSON-RPC API. Holds no keys — read-only.
#[derive(Clone)]
pub struct SolanaRpc {
    client: Client,
    url: String,
}

impl SolanaRpc {
    pub fn new(url: String) -> Result<Self> {
        let client = Client::builder()
            // Public Solana RPCs can stall; a hung request must not block the
            // MCP server indefinitely (project latency budget is <2s).
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self { client, url })
    }

    /// POST a JSON-RPC request and return the `result` field.
    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
        let resp = self
            .client
            .post(&self.url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Solana request {method} failed"))?;
        let json: Value = resp.json().await?;
        if let Some(err) = json.get("error") {
            bail!("Solana RPC {method} error: {err}");
        }
        json.get("result")
            .cloned()
            .context("Solana RPC response missing result")
    }

    /// Native SOL balance in lamports.
    pub async fn sol_balance(&self, address: &str) -> Result<u64> {
        let result = self
            .call("getBalance", json!([address, { "commitment": "confirmed" }]))
            .await?;
        result
            .get("value")
            .and_then(Value::as_u64)
            .context("getBalance returned no lamport value")
    }

    /// Latest confirmed blockhash (for building transactions). Returns the
    /// base58 blockhash string.
    pub async fn latest_blockhash(&self) -> Result<String> {
        let result = self
            .call("getLatestBlockhash", json!([{ "commitment": "confirmed" }]))
            .await?;
        result
            .pointer("/value/blockhash")
            .and_then(Value::as_str)
            .map(str::to_string)
            .context("getLatestBlockhash returned no blockhash")
    }

    /// Decimals for an SPL mint (from `getTokenSupply`). Needed to convert a
    /// human token amount (e.g. "2 USDC") into raw smallest units.
    pub async fn token_decimals(&self, mint: &str) -> Result<u8> {
        let result = self
            .call("getTokenSupply", json!([mint, { "commitment": "confirmed" }]))
            .await?;
        result
            .pointer("/value/decimals")
            .and_then(Value::as_u64)
            .map(|d| d as u8)
            .context("getTokenSupply returned no decimals")
    }

    /// Whether a specific account exists on chain (used to check the recipient's
    /// associated token account before building a transfer).
    pub async fn account_exists(&self, address: &str) -> Result<bool> {
        let result = self
            .call(
                "getAccountInfo",
                json!([address, { "commitment": "confirmed", "encoding": "base64" }]),
            )
            .await?;
        Ok(!result.get("value").map(Value::is_null).unwrap_or(true))
    }

    /// SPL token balance for an owner + mint, summed across accounts.
    /// Raw is smallest units; `decimals` lets callers show a human amount.
    pub async fn token_balance(&self, owner: &str, mint: &str) -> Result<TokenBalance> {
        let result = self
            .call(
                "getTokenAccountsByOwner",
                json!([
                    owner,
                    { "mint": mint },
                    { "commitment": "confirmed", "encoding": "jsonParsed" }
                ]),
            )
            .await?;

        let accounts = result
            .get("value")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut raw: u64 = 0;
        let mut decimals: u8 = 0;
        for acc in &accounts {
            raw += acc
                .pointer("/account/data/parsed/info/tokenAmount/amount")
                .and_then(Value::as_str)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            if let Some(d) = acc
                .pointer("/account/data/parsed/info/tokenAmount/decimals")
                .and_then(Value::as_u64)
            {
                decimals = d as u8;
            }
        }
        Ok(TokenBalance { raw, decimals })
    }
}

/// A token balance: raw smallest units plus the mint's decimals, so callers
/// can present a human-readable amount instead of an opaque raw count.
#[derive(Debug, Clone, Copy)]
pub struct TokenBalance {
    pub raw: u64,
    pub decimals: u8,
}

impl TokenBalance {
    /// Human-readable amount (raw / 10^decimals).
    pub fn ui_amount(&self) -> f64 {
        self.raw as f64 / 10f64.powi(self.decimals as i32)
    }
}