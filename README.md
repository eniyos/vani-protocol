# Vani Protocol (वाणी प्रोटोकॉल)

> **"बोलो, Solana चलेगा"** — Speak, Solana runs.

A Rust-native [Model Context Protocol (MCP)](https://modelcontextprotocol.io) server that lets AI
agents (Claude, Cursor, VS Code) execute Solana DeFi actions from **vernacular voice or text** in
Hindi, Telugu, Tamil, and more — **without any private keys**.

The core idea: India's next 100M crypto users won't use English-first interfaces. Vani is the layer
that lets them *speak*, on chain, securely (MPC signing comes in Week 3).

> Status: **Weeks 1–3 complete (2026-08-07).** Read-only MCP tools over stdio, a live Sarvam AI
> voice layer (STT/TTS), and **Turnkey TEE signing** — native-SOL sends signed off-process, so no
> wallet key ever touches Vani. Token/swap execution and grant docs land in Weeks 3.5–4. This is the
> project's codebase — for the knowledge base that tracks design, decisions, and roadmap, see the
> [Vani-Protocol-brain](https://github.com/eniyos/Vani-Protocol-brain) repo.

## What this builds on

- **rmcp 3.x** — the official Rust MCP SDK (`modelcontextprotocol/rust-sdk`). Tokio-based server
  over **stdio**, tool schemas derived from Rust types via `schemars`.
- **Solana JSON-RPC** — read-only balance/token queries (Devnet by default, zero-cost).
- **Jupiter public APIs** — read-only price + swap-quote lookups (no keys).

## Quick start

```sh
cargo run -p vani-mcp
```

The server speaks MCP over **stdio**. Point any MCP-capable agent at it:

```jsonc
// .claude/settings.json  (Claude Code) or Cursor MCP config
// Run from the repo root, or set "cwd" to your clone's path.
{
  "mcpServers": {
    "vani": {
      "command": "cargo",
      "args": ["run", "-p", "vani-mcp"]
    }
  }
}
```

## Tools (v1 — all read-only, no keys)

| Tool | Description |
|------|-------------|
| `get_balance`       | SOL balance for an address (lamports → SOL) |
| `get_token_balance` | SPL token balance for an address + mint (human amount + raw units) |
| `get_price`         | Live token price (Jupiter price API v3; SOL/USDC/USDT/BONK/JUP) |
| `jupiter_quote`     | Swap quote (Jupiter swap API v1) — no execution |
| `vani_command`      | Parse a Hindi/Hinglish/Telugu/Tamil/English command into a structured intent |
| `tts_speak`         | Text → speech (Sarvam `bulbul:v3`), returns base64 WAV |
| `stt_transcribe`    | Audio → text (Sarvam `saaras:v3`), returns `[language] transcript` |
| `turnkey_create_wallet` | Provision a Solana wallet in Turnkey's TEE; returns wallet id + address |
| `vani_execute`      | Send native SOL (vernacular `"5 SOL bhej do"`), signed in Turnkey's TEE — no key touches Vani |

## Configuration

Copy `.env.example` → `.env`:

| Env var | Purpose | Default |
|---------|---------|---------|
| `RPC_URL` | Solana JSON-RPC endpoint | `https://api.devnet.solana.com` |
| `VANI_DEFAULT_ADDRESS` | Address used when a tool's address field is left empty | none |
| `SARVAM_API_KEY` | Sarvam AI key for `tts_speak`/`stt_transcribe` | none (voice tools error without it) |
| `TURNKEY_ORGANIZATION_ID` | Turnkey org for `vani_execute` | none (execution tools error without it) |
| `TURNKEY_API_PUBLIC_KEY` | Turnkey API key public part (P-256 hex) | none |
| `TURNKEY_API_PRIVATE_KEY` | Turnkey API key private part (32-byte hex) | none |
| `TURNKEY_SOLANA_WALLET_ADDRESS` | Turnkey wallet signer + fee payer | none |
| `VANI_EXECUTE_NETWORK` | CAIP-2 chain for `vani_execute` | `solana:devnet` |

Keep `.env` out of git (it already is).

## Roadmap

- **Wk 1 ✓:** read-only Rust MCP server over stdio, vernacular command parser (27 tests).
- **Wk 2 ✓:** Sarvam AI STT/TTS voice layer (pulled forward to 08-06) + cross-language commands (32 tests).
- **Wk 3 ✓:** Turnkey TEE signing → `vani_execute` native-SOL sends (Knot Wallet verified un-integrable — dormant, unlicensed, API down — and is itself a Turnkey wrapper; see ADR-007 in the brain). SECURITY.md shipped.
- **Wk 3.5:** SPL-token transfer + on-chain Jupiter swap execution.
- **Wk 4:** beta users, landing page, Superteam India grant.

## License

MIT. Open-source by design — proof-of-work for the grant.