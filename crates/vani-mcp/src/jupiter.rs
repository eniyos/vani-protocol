//! Read-only Jupiter helpers: live prices and swap quotes. No keys, no execution.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde_json::Value;

pub const SOL: &str = "So11111111111111111111111111111111111111112";
pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const BONK: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
pub const JUP: &str = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";

/// Map a common symbol to its mainnet mint. Kept small on purpose (MVP);
/// add symbols as integrations land. Keep in sync with the parser's token list.
/// Public so the execution layer can resolve a parser's token symbol to a mint.
pub fn symbol_to_mint(symbol: &str) -> Option<&'static str> {
    match symbol.trim().to_ascii_uppercase().as_str() {
        "SOL" | "WSOL" | "WRAPPED_SOL" => Some(SOL),
        "USDC" => Some(USDC),
        "USDT" => Some(USDT),
        "BONK" => Some(BONK),
        "JUP" => Some(JUP),
        _ => None,
    }
}

/// Live price for one or more symbols (comma-separated). Jupiter Price API v3.
/// Output is in the caller's symbol order (with duplicates dropped), so a voice
/// reply reads back the tokens the user asked for in the order they asked.
pub async fn price(client: &Client, symbols: &str) -> Result<String> {
    let mut mints: Vec<&str> = Vec::new();
    for s in symbols.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let mint = symbol_to_mint(s)
            .context("get_price supports only SOL, USDC, USDT, BONK, JUP")?;
        if !mints.contains(&mint) {
            mints.push(mint);
        }
    }
    if mints.is_empty() {
        bail!("no symbols given");
    }

    let url = format!("https://api.jup.ag/price/v3?ids={}", mints.join(","));
    let resp: Value = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // v3 returns the map directly: { mint: { usdPrice, priceChange24h, ... } }
    let data = resp.as_object().context("unexpected price response")?;
    if data.is_empty() {
        bail!("no price data returned for {symbols}");
    }

    let mut lines = Vec::new();
    for &mint in &mints {
        let info = data.get(mint).context("price data missing for a requested symbol")?;
        let symbol = match mint {
            SOL => "SOL",
            USDC => "USDC",
            USDT => "USDT",
            BONK => "BONK",
            JUP => "JUP",
            _ => &mint[..8.min(mint.len())],
        };
        let p = info.get("usdPrice").and_then(Value::as_f64).unwrap_or(f64::NAN);
        let chg = info.get("priceChange24h").and_then(Value::as_f64).unwrap_or(f64::NAN);
        let price = if p.is_nan() { "n/a".to_string() } else { format!("${p:.6}") };
        let change = if chg.is_nan() { "n/a".to_string() } else { format!("{chg:+.2}%") };
        lines.push(format!("{symbol}: {price} (24h {change})"));
    }
    Ok(lines.join("\n"))
}

/// Read-only swap quote from the Jupiter Swap API v1. `amount` is in the input
/// token's smallest unit (lamports for SOL). Returns a human-readable summary.
pub async fn quote(
    client: &Client,
    input_mint: &str,
    output_mint: &str,
    amount: u64,
) -> Result<String> {
    if amount == 0 {
        bail!("amount must be greater than zero");
    }
    let url = format!(
        "https://api.jup.ag/swap/v1/quote?inputMint={input_mint}&outputMint={output_mint}&amount={amount}"
    );
    let resp: Value = client
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if let Some(err) = resp.get("error") {
        bail!("Jupiter quote error: {err}");
    }

    let out = resp.get("outAmount").and_then(Value::as_str).unwrap_or("?");
    let impact = resp.get("priceImpactPct").and_then(Value::as_str).unwrap_or("?");
    let routes = resp
        .get("routePlan")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);

    Ok(format!(
        "Swap {amount} smallest-units ({input_mint}) -> {out} ({output_mint}) | priceImpact ~{impact}% | {routes} route step(s)"
    ))
}