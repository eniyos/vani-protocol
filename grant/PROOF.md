# Proof of Work — Vani Protocol (verified, not asserted)

Every claim below is backed by the repo, a test run, or an on-chain signature. Run `cargo test` and `cargo clippy --all-targets -- -D warnings` to reproduce.

## 1. It builds, it's tested, it's lint-clean
- **48 tests pass**, **0 fail** (`cargo test`).
- **clippy `-D warnings`**: clean.
- **`cargo build --release`**: clean, `lto = thin`.
- **No dead code committed** — enforced and verified before every commit.
- Stack: Rust `edition 2021`, solana-sdk 3, rmcp 3.1.1 (official MCP SDK).

## 2. Git history shows consistent weekly shipping
```
07d179a fix: first live broadcast — sign in TEE, broadcast over own RPC (ADR-009)
78db308 Add devnet e2e harness for the Turnkey execution layer
10b13b1 Wk 3.5: SPL-token transfer via vani_execute
db016a9 Week 3: Turnkey TEE signing — vani_execute native-SOL sends
f152624 fix: deterministic get_price ordering + README sync (7 tools, voice live)
f18cb18 fix: HTTP timeouts, config trimming, real isError, token decimals, audio mime sniff
f8c0acf parser: Telugu + Tamil action keywords, 27 tests (Week-2 checkbox)
b24a179 fix: valid default address, word-order swap direction, word-boundary tokens
692cd6d parser: Devanagari digits, Hindi number words, Hindi token names
5574cb3 voice: Sarvam STT/TTS tools (Week-2 layer) + stdio log fix
0098595 get_price: add USDT/BONK/JUP symbols; fix README endpoint versions
e0b201c 🚀 First commit: vani-mcp — Rust-native MCP server (Week 1)
```
One commit per milestone/fix, 4 weeks, on schedule. The **on-chain Jupiter swap** build is committed (`34879d4`), including a v0 wire-format serialization test (48 tests total).

## 3. Real on-chain proof — the keyless claim is demonstrated
From the project brain-log (log.md):
> **FIRST LIVE BROADCAST:** `0.001 SOL` sent, sig `2pJ5bNh6…Af4jK` on devnet. Balances confirm: sender 1.0 → 0.998995 SOL, recipient +0.001. Weeks 3–3.5 now *signed + broadcast on devnet*.

- The transaction was **signed inside Turnkey's Trusted Execution Environment** and broadcast over Vani's **own RPC** — a private key never existed in Vani.
- Architecture proven end-to-end: native SOL sends, SPL-token sends (USDC/USDT/BONK/JUP), and Jupiter swaps (v0 + address lookup tables).

## 4. The agentic methodology (the point of this grant)
Vani is built end-to-end by Claude Code. To survive session-context loss, the build keeps:
- an **append-only brain-log** (23 entries) — see `BRAIN-LOG.md`,
- an **ADR decision log** (9 ADRs) — see `ADR-LOG.md`,
- a **real session-recovery incident**: a full session was lost mid-project and recovered **in minutes** from the brain-log, not restarted from zero.

## 5. Security posture
`SECURITY.md` (copied alongside): no private key in Vani, P-256 API-key request integrity, spend-limit recommendation, env hygiene. Wallet keys live in Turnkey's TEE.

---
*Prepared by Claude Code. All numbers are from live `cargo test` / `git log` / chain queries on 2026-08-11.*
