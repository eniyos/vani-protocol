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

use crate::jupiter;
use crate::rpc::SolanaRpc;
use crate::turnkey::TurnkeyClient;
use crate::vanicommand;

/// Approximate Devnet fee budget (lamports) checked before submitting so we
/// don't send Turnkey a transaction that is doomed to fail at broadcast.
const FEE_BUDGET_LAMPORTS: u64 = 5_000;

/// SPL Token program id (classic, not Token-2022).
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
/// Associated Token Account program id — derives a deterministic token account
/// per (owner, mint) so we can transfer without the owner pre-creating one.
const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

/// The associated token account address for `owner` + `mint`: the built-in
/// PDA `find_program_address([owner, token_program, mint], &ATA_PROGRAM_ID)`.
/// Deterministic — no account needs to exist for the address to be derived.
pub fn associated_token_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let token_pid = TOKEN_PROGRAM_ID.parse::<Pubkey>().expect("valid token pid");
    let ata_pid = ASSOCIATED_TOKEN_PROGRAM_ID
        .parse::<Pubkey>()
        .expect("valid ATA pid");
    Pubkey::find_program_address(
        &[owner.as_ref(), token_pid.as_ref(), mint.as_ref()],
        &ata_pid,
    )
    .0
}

/// SPL `TransferChecked` instruction (Token instruction #12): `source`,
/// `mint`, `destination`, `authority` (the signer). Carries the exact amount
/// and decimals so the recipient validates units on chain.
pub fn transfer_checked_ix(
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    decimals: u8,
) -> solana_sdk::instruction::Instruction {
    let token_pid = TOKEN_PROGRAM_ID.parse::<Pubkey>().expect("valid token pid");
    solana_sdk::instruction::Instruction {
        program_id: token_pid,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(*source, false),
            solana_sdk::instruction::AccountMeta::new_readonly(*mint, false),
            solana_sdk::instruction::AccountMeta::new(*destination, false),
            solana_sdk::instruction::AccountMeta::new_readonly(*authority, true),
        ],
        // TransferChecked = discriminant 12, then amount(u64 LE) + decimals(u8).
        data: [&[12u8][..], &amount.to_le_bytes()[..], &[decimals]].concat(),
    }
}

/// Build an unsigned native-SOL transfer transaction for Turnkey to sign.
/// `from` is the fee payer + signer (the Turnkey wallet address). Returns the
/// serialized wire format as hex — signatures are zeroized (unsigned), exactly
/// what Turnkey's `sign_transaction` expects.
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

/// Build an unsigned SPL-token transfer (`TransferChecked`) for Turnkey to
/// sign. `from` is the owning wallet (its associated token account is the
/// source); `to` the recipient owner (its associated token account is the
/// destination). `raw_amount` is in the token's smallest units.
pub async fn build_token_transfer_hex(
    rpc: &SolanaRpc,
    from: &str,
    to: &str,
    mint: &str,
    raw_amount: u64,
    decimals: u8,
) -> Result<String> {
    let from_pk: Pubkey = from.parse().context("invalid owner address")?;
    let to_pk: Pubkey = to.parse().context("invalid recipient address")?;
    let mint_pk: Pubkey = mint.parse().context("invalid mint address")?;
    let blockhash: Hash = rpc
        .latest_blockhash()
        .await?
        .parse()
        .context("invalid recent blockhash from RPC")?;

    let source = associated_token_address(&from_pk, &mint_pk);
    let destination = associated_token_address(&to_pk, &mint_pk);

    // The recipient must have an associated token account to receive; if it
    // doesn't exist yet, surface a clear message instead of a doomed broadcast.
    if !rpc.account_exists(&destination.to_string()).await? {
        bail!(
            "recipient {to} has no associated {mint} token account at {destination} — they need \
             to create it first (any USDC/Bonk deposit or ATA airdrop does)"
        );
    }

    let ix = transfer_checked_ix(
        &source, &mint_pk, &destination, &from_pk, raw_amount, decimals,
    );
    let mut msg = Message::new(&[ix], Some(&from_pk));
    msg.recent_blockhash = blockhash;
    let tx = Transaction::new_unsigned(msg);
    Ok(hex::encode(bincode::serialize(&tx)?))
}

/// Execute a vernacular command as a transfer, signed by Turnkey's TEE.
///
/// Two paths:
/// - `source == "sol"` → native-SOL transfer (system program).
/// - `source` is a known token symbol (USDC/USDT/BONK/JUP) → SPL token
///   transfer (`TransferChecked`) between associated token accounts.
///
/// The amount comes from the explicit `amount` param when given, otherwise from
/// the parsed intent. The signer is the configured Turnkey wallet address.
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

    if intent.action == "swap" {
        bail!("swap execution isn't live yet — only sends for now; swap lands in the next slice");
    }

    // Safety: a send MUST name a known token. If none was detected, fail before
    // any RPC — an unrecognized symbol ("5 ETH bhej do") must never silently
    // fall through to the native-SOL path and send the wrong asset.
    let Some(sym) = intent.source.as_deref() else {
        bail!(
            "I couldn't find a token in that command to send — try \"5 SOL bhej do\" or \"2 USDC bhej do\" \
             (I know SOL, USDC, USDT, BONK, JUP)"
        );
    };

    // A non-SOL token ("2 USDC bhej do") ⇒ SPL token transfer. The amount is
    // resolved once here (explicit param, else the parsed intent) and passed in.
    if sym != "sol" {
        return execute_token(rpc, turnkey, signer, to, amount.or(intent.amount), caip2, sym).await;
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
    // Sign in the TEE, broadcast via our own RPC — avoids the org-gated
    // `SolSendTransaction` broadcast service while keeping keys out of Vani.
    let signed = turnkey.sign_transaction(&unsigned_hex, signer).await?;
    let sig = rpc.send_transaction(&signed).await?;

    Ok(format!(
        "sent {sol} SOL to {to} · signature {sig} · {caip2} (signed in Turnkey TEE, broadcast via RPC — no key touched Vani)"
    ))
}

/// SPL-token send path: resolves the symbol to a mint, converts the human
/// amount to raw units using the on-chain decimals, and hands a
/// `TransferChecked` transaction to Turnkey's TEE to sign + broadcast.
async fn execute_token(
    rpc: &SolanaRpc,
    turnkey: &TurnkeyClient,
    signer: &str,
    to: &str,
    amount: Option<f64>,
    caip2: &str,
    sym: &str,
) -> Result<String> {
    let mint = jupiter::symbol_to_mint(sym)
        .context("unknown token — I know USDC, USDT, BONK, JUP (SOL is native)")?;

    let ui = amount.context("I need an amount to send — e.g. \"2 USDC bhej do\"")?;
    if ui.is_nan() || ui <= 0.0 {
        bail!("amount must be a positive number, got {ui}");
    }

    let decimals = rpc.token_decimals(mint).await?;
    let raw = (ui * 10f64.powi(decimals as i32)).round() as u64;
    if raw == 0 {
        bail!("amount too small — {ui} {sym} is less than one {}-unit", 10i64.pow(decimals as u32));
    }

    // Safety: verify the sender actually holds that many units.
    let held = rpc.token_balance(signer, mint).await?;
    if held.raw < raw {
        let avail = held.ui_amount();
        bail!(
            "insufficient {sym}: {avail:.6} available, need ≥{ui} ({raw} raw units, {}-decimals)",
            decimals
        );
    }

    let unsigned_hex = build_token_transfer_hex(rpc, signer, to, mint, raw, decimals).await?;
    // Sign in the TEE, broadcast via our own RPC (same rationale as the SOL path).
    let signed = turnkey.sign_transaction(&unsigned_hex, signer).await?;
    let sig = rpc.send_transaction(&signed).await?;

    Ok(format!(
        "sent {ui} {sym} to {to} · signature {sig} · {caip2} (signed in Turnkey TEE, broadcast via RPC — no key touched Vani)"
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
    fn unknown_symbol_rejected_before_network() {
        // "5 ETH bhej do" — ETH isn't a known token, so no source is detected.
        // The send must fail with a clear message BEFORE any RPC (it must never
        // fall through and silently send native SOL).
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
                None, "5 ETH bhej do", "solana:devnet",
            ))
            .unwrap_err();
        assert!(err.to_string().contains("couldn't find a token"), "got: {err}");
    }

    #[test]
    fn associated_token_address_is_deterministic_pda() {
        let owner = "GkmtF3gxR6DB4YpKcB3ZLx6ZXg9xSxQmB4LYQzKqTm3L"
            .parse::<Pubkey>()
            .unwrap();
        let mint = jupiter::USDC.parse::<Pubkey>().unwrap();
        let ata = associated_token_address(&owner, &mint);
        // Deterministic: same inputs ⇒ same address.
        assert_eq!(ata, associated_token_address(&owner, &mint));
        // An ATA is a program-derived address — it must be off the ed25519
        // curve (a PDA can never be a normal keypair address).
        assert!(!ata.is_on_curve(), "ATA should be a PDA (off-curve): {ata}");
        // Different owner ⇒ different ATA.
        let other = "4Nd1m2LsN7A7T8QzHkHjYdE5E4F9uGpXpWcDvMqRbZa1"
            .parse::<Pubkey>()
            .unwrap();
        assert_ne!(ata, associated_token_address(&other, &mint));
    }

    #[test]
    fn transfer_checked_instruction_has_correct_bytes() {
        let src = associated_token_address(
            &"GkmtF3gxR6DB4YpKcB3ZLx6ZXg9xSxQmB4LYQzKqTm3L"
                .parse::<Pubkey>()
                .unwrap(),
            &jupiter::USDC.parse::<Pubkey>().unwrap(),
        );
        let dst = associated_token_address(
            &"4Nd1m2LsN7A7T8QzHkHjYdE5E4F9uGpXpWcDvMqRbZa1"
                .parse::<Pubkey>()
                .unwrap(),
            &jupiter::USDC.parse::<Pubkey>().unwrap(),
        );
        let mint = jupiter::USDC.parse::<Pubkey>().unwrap();
        let authority = "11111111111111111111111111111111".parse::<Pubkey>().unwrap();

        let ix = transfer_checked_ix(&src, &mint, &dst, &authority, 1_250_000, 6);
        // TransferChecked = discriminant 12 (0x0c), then amount u64 LE, then u8 decimals.
        // 1,250,000 = 0x001312D0 → LE bytes D0 12 13 00 00 00 00 00.
        assert_eq!(ix.data, vec![12, 0xD0, 0x12, 0x13, 0, 0, 0, 0, 0, 6]);
        // Program is the SPL Token program; authority is the only signer.
        assert_eq!(ix.program_id.to_string(), TOKEN_PROGRAM_ID);
        assert_eq!(ix.accounts.len(), 4);
        assert!(ix.accounts[3].is_signer, "authority must sign");
        assert!(!ix.accounts[0].is_signer && !ix.accounts[1].is_signer && !ix.accounts[2].is_signer);
    }

    #[test]
    fn token_path_routes_usdc_to_token_send_not_sol() {
        // "2 USDC bhej do" must route to the SPL-token path: its first RPC is
        // getTokenSupply (for decimals), NOT getBalance. On localhost:1 the
        // request fails, but the error must prove which path was taken.
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
                None, "2 USDC bhej do", "solana:devnet",
            ))
            .unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("getTokenSupply"), "expected token path, got: {msg}");
        assert!(!msg.contains("getBalance"), "must not hit SOL balance: {msg}");
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
