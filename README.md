# वाणी · Vani Protocol

[![CI](https://github.com/eniyos/vani-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/eniyos/vani-protocol/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/vani-mcp.svg)](https://crates.io/crates/vani-mcp)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **बोलो, Solana चलेगा** — Speak Hindi, Telugu, or Tamil. Solana runs it.

Vani is a **Rust-native [MCP](https://modelcontextprotocol.io) server** that gives any AI agent (Claude, Cursor, Kiro) nine Solana tools — live prices, swap quotes, voice transcription, and **on-chain execution** — all driven by natural language in Indian languages. Wallet keys never touch Vani; every transaction is signed inside [Turnkey's](https://turnkey.com) TEE.

```
You  →  "1 SOL se USDC swap karo"
Agent → vani_execute({ text: "1 SOL se USDC swap karo" })
Vani → Jupiter quote → Turnkey signs → devnet broadcast → sig: 2pJ5bNh6…
```

---

## Install

### Option A — cargo (requires Rust)
```sh
cargo install vani-mcp
```

### Option B — pre-built binary (no Rust needed)
Download the archive for your platform from the [latest release](https://github.com/eniyos/vani-protocol/releases/latest), extract, and place `vani-mcp` on your `$PATH`.

| Platform | File |
|----------|------|
| macOS Apple Silicon | `vani-mcp-*-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `vani-mcp-*-x86_64-apple-darwin.tar.gz` |
| Linux x86_64 (static) | `vani-mcp-*-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 | `vani-mcp-*-aarch64-unknown-linux-musl.tar.gz` |
| Windows | `vani-mcp-*-x86_64-pc-windows-msvc.zip` |

### Option C — npx (no install at all)
```sh
npx vani-mcp
```

---

## Wire into your AI agent

Add this block to your MCP client config and restart the client.

**Claude Desktop** — `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Cursor** — `.cursor/mcp.json`  
**Kiro** — `~/.kiro/settings/mcp.json`

```jsonc
{
  "mcpServers": {
    "vani": {
      "command": "vani-mcp",        // or full path to the binary
      "env": {
        // ── Voice (optional) ──────────────────────────────────────────────
        "SARVAM_API_KEY": "sk_...",          // sarvam.ai → free ₹100 credits

        // ── On-chain execution (optional) ─────────────────────────────────
        "TURNKEY_ORGANIZATION_ID": "...",    // turnkey.com → free Starter org
        "TURNKEY_API_PUBLIC_KEY":  "...",    // P-256 API key (66-char hex)
        "TURNKEY_API_PRIVATE_KEY": "...",    // P-256 API key (64-char hex)
        "TURNKEY_SOLANA_WALLET_ADDRESS": "...", // from turnkey_create_wallet

        // ── Chain (optional) ──────────────────────────────────────────────
        "VANI_EXECUTE_NETWORK": "solana:mainnet-beta",  // default: devnet
        "RPC_URL": "https://api.mainnet-beta.solana.com"
      }
    }
  }
}
```

> **Read-only tools** (`get_balance`, `get_price`, `jupiter_quote`, `vani_command`) work with **zero credentials** — just the binary.  
> **Voice tools** need `SARVAM_API_KEY`.  
> **Execution tools** need all four `TURNKEY_*` vars.

---

## Tools

| Tool | Needs creds? | What it does |
|------|:---:|------|
| `get_balance` | — | SOL balance of any address |
| `get_token_balance` | — | SPL token balance (USDC / USDT / BONK / JUP) |
| `get_price` | — | Live USD price via Jupiter (SOL, USDC, USDT, BONK, JUP) |
| `jupiter_quote` | — | Read-only swap quote — route, output amount, price impact |
| `vani_command` | — | Parse a vernacular command → structured JSON intent |
| `tts_speak` | Sarvam | Text → base64 WAV (Hindi, Telugu, Tamil + 8 more) |
| `stt_transcribe` | Sarvam | Audio → `[detected-lang] transcript` |
| `turnkey_create_wallet` | Turnkey | Provision a new Solana wallet in Turnkey's TEE |
| `vani_execute` | Turnkey | **Send SOL / SPL token or swap** — signed in TEE, keys never leave Turnkey |

---

## Language support

`vani_command` and `vani_execute` understand commands in:

| Language | Example |
|----------|---------|
| Hindi | `"एक SOL स्वैप करो USDC में"` |
| Hinglish | `"1 SOL se USDC swap karo"` |
| Telugu | `"1 SOL ని USDC కి మార్చు"` |
| Tamil | `"1 SOL ஐ USDC ஆக மாற்று"` |
| English | `"swap 1 SOL for USDC"` |

Numbers work in Devanagari digits (१, २, …) and Hindi words (एक, दो, तीन…).

---

## Quickstart — first execution in 5 minutes

```sh
# 1. Install
cargo install vani-mcp

# 2. Sign up (both free):
#    Sarvam AI  → https://dashboard.sarvam.ai   (get SARVAM_API_KEY)
#    Turnkey    → https://app.turnkey.com        (get org id + generate API key pair)

# 3. Create your on-chain wallet (once)
#    In your agent: call turnkey_create_wallet({ name: "my-vani-wallet" })
#    → copy the returned address → set TURNKEY_SOLANA_WALLET_ADDRESS

# 4. Fund it (devnet — free)
solana airdrop 1 <YOUR_WALLET_ADDRESS> --url devnet

# 5. Tell your agent:
#    "0.01 SOL bhej do" (to: <any-devnet-address>)
#    → Vani parses, builds tx, Turnkey TEE signs, broadcasts, returns tx signature
```

---

## Architecture

```
AI Agent (Claude / Cursor / Kiro)
        │  JSON-RPC over stdio (MCP)
        ▼
   vani-mcp  ←── server.rs (9 tools)
        │
        ├── vanicommand.rs   rule-based NLU for 5 languages
        ├── jupiter.rs       price API v3 + swap API v1
        ├── rpc.rs           Solana JSON-RPC (balance, broadcast)
        ├── sarvam.rs        STT saaras:v3  /  TTS bulbul:v3
        ├── execute.rs       unsigned tx builder (SOL / SPL / swap)
        └── turnkey.rs       P-256 stamped requests → TEE sign
```

**Security invariant:** Vani constructs unsigned transactions locally and hands bytes to Turnkey's enclave for signing. A compromised Vani process can *request* a transfer; it can never exfiltrate a wallet key.

---

## Configuration reference

| Env var | Default | Purpose |
|---------|---------|---------|
| `RPC_URL` | `https://api.devnet.solana.com` | Solana JSON-RPC endpoint |
| `VANI_DEFAULT_ADDRESS` | *(none)* | Fallback address when a tool's address field is empty |
| `SARVAM_API_KEY` | *(none)* | Sarvam AI key — voice tools error gracefully without it |
| `TURNKEY_ORGANIZATION_ID` | *(none)* | Turnkey org — execution tools error gracefully without it |
| `TURNKEY_API_PUBLIC_KEY` | *(none)* | P-256 API key public part (66-char hex, `02`/`03` prefix) |
| `TURNKEY_API_PRIVATE_KEY` | *(none)* | P-256 API key private part (64-char hex) |
| `TURNKEY_SOLANA_WALLET_ADDRESS` | *(none)* | Signer + fee payer — output of `turnkey_create_wallet` |
| `VANI_EXECUTE_NETWORK` | `solana:devnet` | CAIP-2 chain for `vani_execute` |

Copy `.env.example` → `.env` to get started.

---

## Development

```sh
git clone https://github.com/eniyos/vani-protocol
cd vani-protocol
cp .env.example .env   # fill in your keys

cargo test             # 48 unit tests
cargo clippy -p vani-mcp --all-targets -- -D warnings
cargo run -p vani-mcp  # start the MCP server over stdio
```

---

## License

MIT — open-source by design.
