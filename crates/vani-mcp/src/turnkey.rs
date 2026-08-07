//! Turnkey TEE signing client — the Week-3 execution layer.
//!
//! Vani never holds, sees, or transmits a wallet private key: keys live inside
//! Turnkey's Trusted Execution Environment. We authenticate each request with
//! an API key (ECDSA P-256 over the exact JSON body) and ask Turnkey to sign a
//! Solana transaction and broadcast it (`sol_send_transaction`).
//!
//! Wire protocol (verified against `tkhq/sdk` source):
//! - Header `X-Stamp: base64url({"publicKey","scheme":"SIGNATURE_SCHEME_TK_API_P256",
//!   "signature":<DER-hex>})` where the signature covers the exact request body.
//! - Submit methods: `POST /public/v1/submit/<method>` with
//!   `{type, timestampMs, organizationId, parameters}`.
//! - Queries: `POST /public/v1/query/<method>` with the plain params object.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use base64::Engine;
use p256::ecdsa::{Signature, SigningKey, signature::hazmat::PrehashSigner};
use reqwest::Client;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Turnkey's hosted API base.
const DEFAULT_BASE: &str = "https://api.turnkey.com";

/// A freshly-created Solana wallet in Turnkey: its id plus the first derived
/// address (used as signer + fee payer for transactions).
pub struct CreatedWallet {
    pub wallet_id: String,
    pub address: String,
}

/// Thin client over Turnkey's public API. The `signer` is the API key's EC
/// private key, used only to stamp requests — never a wallet key.
#[derive(Clone)]
pub struct TurnkeyClient {
    http: Client,
    base: String,
    org_id: String,
    /// Compressed P-256 public key hex, echoed in the auth stamp.
    public_hex: String,
    /// API-key private key (32-byte raw scalar), used to stamp request bodies.
    signer: SigningKey,
}

impl TurnkeyClient {
    /// Build the client from the Turnkey API key pair. `api_public_hex` is the
    /// compressed P-256 public key hex; `api_private_hex` the raw 32-byte
    /// private key hex. Both come from the Turnkey dashboard (gitignored env).
    pub fn new(
        http: Client,
        org_id: String,
        api_public_hex: String,
        api_private_hex: String,
    ) -> Result<Self> {
        let bytes = hex::decode(api_private_hex.trim())
            .context("TURNKEY_API_PRIVATE_KEY must be hex")?;
        let scalar: [u8; 32] = bytes.try_into().map_err(|_| {
            anyhow::anyhow!("TURNKEY_API_PRIVATE_KEY must be exactly 32 bytes (64 hex chars)")
        })?;
        let signer = SigningKey::from_bytes((&scalar).into())
            .context("TURNKEY_API_PRIVATE_KEY is not a valid P-256 scalar")?;
        Ok(Self {
            http,
            base: DEFAULT_BASE.to_string(),
            org_id,
            public_hex: api_public_hex,
            signer,
        })
    }

    /// Sign the request body with the API key and return the `X-Stamp` value:
    /// base64url of `{publicKey, scheme, signature(DER hex)}`.
    fn stamp(&self, body: &str) -> Result<String> {
        let hash = Sha256::digest(body.as_bytes());
        let sig: Signature = self.signer.sign_prehash(&hash)?;
        let stamp = json!({
            "publicKey": self.public_hex,
            "scheme": "SIGNATURE_SCHEME_TK_API_P256",
            "signature": hex::encode(sig.to_der().as_bytes()),
        });
        Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(stamp.to_string().as_bytes()))
    }

    /// POST a JSON body to a Turnkey path, stamped with the API key. Returns
    /// the parsed JSON on success; surfaces Turnkey's error message otherwise.
    async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let body_str = serde_json::to_string(body)?;
        let x_stamp = self.stamp(&body_str)?;
        let resp = self
            .http
            .post(format!("{}{}", self.base, path))
            .header("Content-Type", "application/json")
            .header("X-Stamp", x_stamp)
            .body(body_str)
            .send()
            .await
            .with_context(|| format!("Turnkey {path} request failed"))?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            bail!("Turnkey {path} {status}: {text}");
        }
        let value: Value = serde_json::from_str(&text)
            .with_context(|| format!("Turnkey {path} returned non-JSON: {text}"))?;
        if let Some(msg) = value.pointer("/error/message").and_then(Value::as_str) {
            bail!("Turnkey {path} error: {msg}");
        }
        Ok(value)
    }

    /// Epoch-millisecond timestamp (string) required by submit envelopes.
    fn timestamp_ms(&self) -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .to_string()
    }

    /// Standard submit envelope: `{type, timestampMs, organizationId, parameters}`.
    fn submit_body(&self, activity_type: &str, parameters: Value) -> Value {
        json!({
            "type": activity_type,
            "timestampMs": self.timestamp_ms(),
            "organizationId": self.org_id,
            "parameters": parameters,
        })
    }

    /// Provision a new Solana wallet in Turnkey and return its id + first
    /// derived address. One call: `create_wallet` derives the account.
    pub async fn create_wallet(&self, name: &str) -> Result<CreatedWallet> {
        let parameters = json!({
            "walletName": name,
            "mnemonicLength": 12,
            "accounts": [{
                "curve": "CURVE_ED25519",
                "pathFormat": "PATH_FORMAT_BIP32",
                "path": "m/44'/501'/0'/0'",
                "addressFormat": "ADDRESS_FORMAT_SOLANA",
            }],
        });
        let resp = self
            .post(
                "/public/v1/submit/create_wallet",
                &self.submit_body("ACTIVITY_TYPE_CREATE_WALLET", parameters),
            )
            .await?;

        let wallet_id = resp
            .pointer("/activity/result/walletId")
            .and_then(Value::as_str)
            .context("create_wallet activity missing walletId")?
            .to_string();
        let address = resp
            .pointer("/activity/result/addresses/0")
            .and_then(Value::as_str)
            .context("create_wallet activity missing derived address")?
            .to_string();
        Ok(CreatedWallet { wallet_id, address })
    }

    /// Ask Turnkey to sign an unsigned Solana transaction (wire format, hex)
    /// with `signer` and broadcast it. Returns the on-chain transaction
    /// signature. Polls the send status a few times for the signature.
    pub async fn send_sol_transaction(
        &self,
        unsigned_transaction_hex: &str,
        signer: &str,
        caip2: &str,
    ) -> Result<String> {
        let parameters = json!({
            "unsignedTransaction": unsigned_transaction_hex,
            "signWiths": [signer],
            "sponsor": false,
            "caip2": caip2,
        });
        let resp = self
            .post(
                "/public/v1/submit/sol_send_transaction",
                &self.submit_body("ACTIVITY_TYPE_SOL_SEND_TRANSACTION_V2", parameters),
            )
            .await?;

        let status_id = resp
            .pointer("/activity/result/sendTransactionStatusId")
            .and_then(Value::as_str)
            .context("sol_send_transaction activity missing sendTransactionStatusId")?
            .to_string();

        // Poll the send status until the signature is available or a terminal
        // failure surfaces (Turnkey broadcasts async; ~2s budget like the rest
        // of the server).
        for _ in 0..5 {
            let query = json!({
                "organizationId": self.org_id,
                "sendTransactionStatusId": status_id,
            });
            let status = self
                .post("/public/v1/query/get_send_transaction_status", &query)
                .await?;
            if let Some(sig) = status.pointer("/solana/signature").and_then(Value::as_str) {
                return Ok(sig.to_string());
            }
            if let Some(err) = status.get("txError").and_then(Value::as_str) {
                bail!("transaction broadcast failed: {err}");
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        bail!("timed out waiting for Turnkey to broadcast the transaction")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed valid P-256 scalar (deterministic, test-only).
    fn test_client() -> TurnkeyClient {
        let http = Client::new();
        TurnkeyClient::new(
            http,
            "org-test".into(),
            "03".repeat(32), // any 66-char hex public key — echoed, not verified
            "01".repeat(32),
        )
        .expect("test client")
    }

    #[test]
    fn stamp_is_base64url_of_expected_json() {
        let c = test_client();
        let body = r#"{"type":"ACTIVITY_TYPE_CREATE_WALLET"}"#;
        let stamp = c.stamp(body).expect("stamp");
        // Must be URL-safe base64 without padding (Node's stringToBase64urlString).
        assert!(!stamp.contains('+') && !stamp.contains('/') && !stamp.contains('='));
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&stamp)
            .expect("stamp decodes");
        let v: Value = serde_json::from_slice(&decoded).expect("stamp is JSON");
        assert_eq!(v["scheme"], "SIGNATURE_SCHEME_TK_API_P256");
        assert_eq!(v["publicKey"], c.public_hex);
        let sig = v["signature"].as_str().expect("signature hex");
        // DER ECDSA signatures start with the SEQUENCE tag 0x30.
        assert!(sig.starts_with("30"), "expected DER, got {sig}");
        // DER P-256 signatures are 70..=72 bytes depending on r/s high bits.
        let len = hex::decode(sig).expect("signature is hex").len();
        assert!((70..=72).contains(&len), "unexpected DER length {len}");
    }

    #[test]
    fn stamp_is_stable_per_body() {
        let c = test_client();
        // Same body ⇒ same signature (deterministic RFC6979), so the stamp is
        // reproducible — useful to assert request integrity.
        assert_eq!(c.stamp("hello").unwrap(), c.stamp("hello").unwrap());
    }

    #[test]
    fn create_wallet_envelope_shape() {
        let c = test_client();
        let body = c.submit_body(
            "ACTIVITY_TYPE_CREATE_WALLET",
            json!({ "walletName": "vani", "mnemonicLength": 12 }),
        );
        assert_eq!(body["type"], "ACTIVITY_TYPE_CREATE_WALLET");
        assert_eq!(body["organizationId"], "org-test");
        assert!(body["timestampMs"].is_string());
        assert!(body["parameters"]["walletName"] == "vani");
    }
}
