//! Execution layer (Week 3): builds an unsigned native-SOL transfer from a
//! vernacular command and asks Turnkey's TEE to sign + broadcast it. Vani
//! never touches a wallet key — it constructs the transaction locally and
//! hands the bytes to Turnkey for signing.

use anyhow::{bail, Context, Result};
use solana_sdk::hash::Hash;
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_system_interface::instruction as system_instruction;

use crate::rpc::SolanaRpc;
use crate::turnkey::TurnkeyClient;
use crate::vanicommand;

/// Approximate Devnet fee budget (lamports) checked before submitting so we
/// don't send Turnkey a transaction that is doomed to fail at broadcast.
const FEE_BUDGET_LAMPORTS: u64 = 5_000;

/// Build an unsigned native-SOL transfer transaction for Turnkey to sign.
/// `from` is the fee payer + signer (the Turnkey wallet address). Returns the
/// serialized wire format as hex — signatures are zeroed (unsigned), exactly
/// what Turnkey's `sol_send_transaction` expects.
pub async fn build_transfer_hex(
    rpc: &SolanaRpc,
    from: &str,
    to: &str,
    lamports: u64,
) -> Result<String> {
    let from_pk: Pubkey = from.parse().context("invalid sender address")?;
    let to_pk: Pubkey = to.parse().context("invalid recipient address")?;
    let blockhash: Hash = rpc
        .latest_blockhash()
        .await?
        .parse()
        .context("invalid recent blockhash from RPC")?;

    let ix = system_instruction::transfer(&from_pk, &to_pk, lamports);
    let mut msg = Message::new(&[ix], Some(&from_pk));
    // `Message::new` leaves the recent blockhash zeroed — set it so the signed
    // transaction is actually valid on chain.
    msg.recent_blockhash = blockhash;
    let tx = Transaction::new_unsigned(msg);
    Ok(hex::encode(bincode::serialize(&tx)?))
}

/// Execute a vernacular command as a native-SOL send, signed by Turnkey.
///
/// MVP scope: only SOL transfers (token + swap execution are documented
/// follow-ups). The amount comes from the explicit `amount` param when given,
/// otherwise from the parsed intent. The signer is the configured Turnkey
/// wallet address.
pub async fn execute(
    rpc: &SolanaRpc,
    turnkey: &TurnkeyClient,
    signer: &str,
    to: &str,
    amount: Option<f64>,
    text: &str,
    caip2: &str,
) -> Result<String> {
    let intent = vanicommand::parse(text);

    // Only native-SOL sends on the MVP execution path — be honest about scope.
    if intent.action == "swap" {
        bail!("swap execution isn't live yet — only native SOL sends for now");
    }
    if intent.source.as_deref().is_some_and(|s| s != "sol") {
        bail!(
            "only native SOL sends are supported right now (command mentions {:?}); token/swap execution is next",
            intent.source
        );
    }

    let sol = amount
        .or(intent.amount)
        .context("I need an amount to send — e.g. \"5 SOL bhej do\"")?;
    if sol.is_nan() || sol <= 0.0 {
        bail!("amount must be a positive number, got {sol}");
    }
    let lamports = (sol * 1e9).round() as u64;

    // Safety: don't submit a transaction the wallet can't pay for.
    let balance = rpc.sol_balance(signer).await?;
    if balance < lamports + FEE_BUDGET_LAMPORTS {
        bail!(
            "insufficient funds: {balance} lamports available, need ≥{} (transfer + fee)",
            lamports + FEE_BUDGET_LAMPORTS
        );
    }

    let unsigned_hex = build_transfer_hex(rpc, signer, to, lamports).await?;
    let sig = turnkey
        .send_sol_transaction(&unsigned_hex, signer, caip2)
        .await?;

    Ok(format!(
        "sent {sol} SOL to {to} · signature {sig} · {caip2} (signed in Turnkey TEE — no key touched Vani)"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_swap_intent() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        // No network needed: validation fails before any I/O.
        let rpc = SolanaRpc::new("http://127.0.0.1:1".into()).unwrap();
        let turnkey = TurnkeyClient::new(
            reqwest::Client::new(),
            "org".into(),
            "03".repeat(32),
            "01".repeat(32),
        )
        .unwrap();
        let err = runtime
            .block_on(execute(
                &rpc, &turnkey, "11111111111111111111111111111111", "22222222222222222222222222222222",
                None, "1 SOL USDC mein swap karo", "solana:devnet",
            ))
            .unwrap_err();
        assert!(err.to_string().contains("swap execution isn't live"));
    }

    #[test]
    fn rejects_non_sol_token() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let rpc = SolanaRpc::new("http://127.0.0.1:1".into()).unwrap();
        let turnkey = TurnkeyClient::new(
            reqwest::Client::new(),
            "org".into(),
            "03".repeat(32),
            "01".repeat(32),
        )
        .unwrap();
        let err = runtime
            .block_on(execute(
                &rpc, &turnkey, "11111111111111111111111111111111", "22222222222222222222222222222222",
                None, "5 USDC bhej do", "solana:devnet",
            ))
            .unwrap_err();
        assert!(err.to_string().contains("only native SOL sends"));
    }

    #[test]
    fn rejects_missing_amount() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let rpc = SolanaRpc::new("http://127.0.0.1:1".into()).unwrap();
        let turnkey = TurnkeyClient::new(
            reqwest::Client::new(),
            "org".into(),
            "03".repeat(32),
            "01".repeat(32),
        )
        .unwrap();
        let err = runtime
            .block_on(execute(
                &rpc, &turnkey, "11111111111111111111111111111111", "22222222222222222222222222222222",
                None, "SOL bhej do", "solana:devnet",
            ))
            .unwrap_err();
        assert!(err.to_string().contains("amount"));
    }
}
