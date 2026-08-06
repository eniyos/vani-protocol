# Vani Protocol (वाणी प्रोटोकॉल)

> **"बोलो, Solana चलेगा"** — Speak, Solana runs.

A Rust-native [Model Context Protocol (MCP)](https://modelcontextprotocol.io) server that lets AI
agents (Claude, Cursor, VS Code) execute Solana DeFi actions from **vernacular voice or text** in
Hindi, Telugu, Tamil, and more — **without any private keys**.

The core idea: India's next 100M crypto users won't use English-first interfaces. Vani is the layer
that lets them *speak*, on chain, securely (MPC signing comes in Week 3).

> Status: **Week 1 foundation.** Read-only MCP tools over stdio. Signing, voice, and grant
> documentation land in later weeks. This is the project's codebase — for the knowledge base that
> tracks design, decisions, and roadmap, see the [Vani-Protocol-brain](https://github.com/eniyos/Vani-Protocol-brain) repo.

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
{
  "mcpServers": {
    "vani": {
      "command": "cargo",
      "args": ["run", "-p", "vani-mcp"],
      "cwd": "/Users/enjo/Vani-Protocol"
    }
  }
}
```

## Tools (v1 — all read-only, no keys)

| Tool | Description |
|------|-------------|
| `get_balance`       | SOL balance for an address (lamports → SOL) |
| `get_token_balance` | SPL token balance for an address + mint |
| `get_price`         | Live token price (Jupiter price API v2) |
| `jupiter_quote`     | Swap quote (Jupiter quote API v6) — no execution |
| `vani_command`      | Parse a Hindi/Hinglish command into a structured intent |

## Configuration

Copy `.env.example` → `.env` and set `RPC_URL` (defaults to `https://api.devnet.solana.com`).

## Roadmap

- **Wk 1 (now):** read-only Rust MCP server over stdio, vernacular command parser.
- **Wk 2:** Sarvam AI STT/TTS voice layer + wiring voice into the tools.
- **Wk 3:** Knot MPC signing → execution.
- **Wk 4:** beta users, landing page, Superteam India grant.

## License

MIT. Open-source by design — proof-of-work for the grant.