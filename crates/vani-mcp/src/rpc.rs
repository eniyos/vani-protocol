//! Minimal Solana JSON-RPC client over any configured endpoint (reqwest).

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
        let client = Client::builder().build()?;
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

    /// SPL token balance (smallest units) for an owner + mint, summed across accounts.
    pub async fn token_balance(&self, owner: &str, mint: &str) -> Result<u64> {
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

        let total: u64 = accounts
            .iter()
            .filter_map(|acc| {
                acc.pointer("/account/data/parsed/info/tokenAmount/amount")
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .sum();

        Ok(total)
    }
}