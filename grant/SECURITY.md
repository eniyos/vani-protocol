# Security

Vani's security guarantee is the same one that anchors its grant story: **no
private key ever exists inside Vani.** Wallets are custodied by [Turnkey](https://turnkey.com)
inside a Trusted Execution Environment (TEE) and signed on request. Vani
constructs unsigned transactions locally and hands the bytes to Turnkey — it
never sees, stores, logs, or transmits a wallet key.

## Key management

| Asset | Where it lives | Notes |
|---|---|---|
| Wallet private keys | **Turnkey TEE** | Derived in a Turnkey sub-org from a wallet seed; the seed never leaves the enclave. Vani only ever references the derived **address**. |
| Turnkey API private key (`TURNKEY_API_PRIVATE_KEY`) | Vani's `.env` | Used to **stamp** requests (ECDSA P-256 over the body) so Turnkey trusts the caller. This is an *API credential*, not a wallet key. |
| Turnkey API public key + org id | Vani's `.env` | Identifiers for the same credential. |
| Sarvam AI key (`SARVAM_API_KEY`) | Vani's `.env` | Voice STT/TTS only; per-request header, never stored or logged. |

All credentials are loaded once at startup from gitignored `.env`, trimmed, and
held in memory. Nothing is written to disk, logs, or any external service beyond
the per-request API call it authorizes.

## Threat model & controls

- **No wallet keys in Vani** — signing is delegated to Turnkey's TEE
  (`sol_send_transaction`). A compromised Vani process can *request* a transfer
  but cannot exfiltrate a key.
- **Request integrity** — every Turnkey call carries an `X-Stamp` signature over
  the exact request body, so requests can't be replayed or tampered in flight.
- **Spend limits (recommended)** — create an [approval policy](https://docs.turnkey.com)
  in the Turnkey dashboard that caps daily spend / whitelists recipients for the
  Vani API key. Vani also performs a local balance check before submitting so it
  never asks Turnkey to broadcast a transaction the wallet can't pay for.
- **Devnet by default** — `VANI_EXECUTE_NETWORK=solana:devnet`. Switch to
  `solana:mainnet` only with a spend-limit policy in place.
- **Latency & timeouts** — all upstream calls (RPC, Jupiter, Sarvam, Turnkey)
  have explicit timeouts so a hung upstream can't wedge the MCP server.

## Environment hygiene

```sh
cp .env.example .env   # then fill in
chmod 600 .env
```

- `.env` is gitignored. Never commit it.
- Never paste `TURNKEY_API_PRIVATE_KEY` or `SARVAM_API_KEY` into chat, issues,
  logs, or docs.
- Rotation: generate a new API key pair in the Turnkey dashboard, replace the
  `.env` values, restart. Old keys can be revoked there.

## Reporting a vulnerability

Open an issue in this repository or email the maintainers directly. This is an
MVP; treat funds as test funds until Mainnet hardening (Weeks 4+).
